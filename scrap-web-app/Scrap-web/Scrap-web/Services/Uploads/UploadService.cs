using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Options;
using Scrap_web.Configuration;
using Scrap_web.Data;
using Scrap_web.DataTransferObjects.Uploads;
using Scrap_web.Models.Uploads;
using Scrap_web.Models.Videos;
using Scrap_web.Services.VideoProcessing;
using Scrap_web.Storage;

namespace Scrap_web.Services.Uploads
{
    public sealed class UploadService : IUploadService
    {
        private readonly ApplicationDbContext _dbContext;
        private readonly IVideoStorage _storage;
        private readonly VideoUploadConfiguration _uploadOptions;
        private readonly IUploadLockManager _uploadLockManager;
        private readonly IBackgroundVideoProcesingQueue _videoProcessingQueue;

        public UploadService(
            ApplicationDbContext dbContext,
            IVideoStorage storage,
            IOptions<VideoUploadConfiguration> uploadOptions,
            IUploadLockManager uploadLockManager,
            IBackgroundVideoProcesingQueue videoProcessingQueue)
        {
            _dbContext = dbContext;
            _storage = storage;
            _uploadOptions = uploadOptions.Value;
            _uploadLockManager = uploadLockManager;
            _videoProcessingQueue = videoProcessingQueue;
        }

        public async Task<CreateUploadResponse> CreateUploadAsync(
            string userId,
            CreateUploadRequest request,
            CancellationToken cancellationToken = default)
        {
            ValidateRequest(request);

            var videoId = Guid.NewGuid();
            var uploadId = Guid.NewGuid();
            var now = DateTime.UtcNow;

            var video = new Video
            {
                Id = videoId,
                OwnerId = userId,
                Title = request.Title.Trim(),
                Description = request.Description?.Trim(),
                FileName = Path.GetFileName(request.FileName),
                ContentType = request.ContentType,
                FileSize = request.FileSize,
                Status = VideoStatus.Processing,
                CreatedAt = now,
            };

            var upload = new Upload
            {
                Id = uploadId,
                VideoId = videoId,
                OwnerId = userId,
                Method = request.Method,
                Status = UploadStatus.Created,
                TotalBytes = request.FileSize,
                UploadedBytes = 0,
                ChunkSize = request.ChunkSize,
                TotalChunks = request.Method == UploadMethod.Chunked
                    ? CalculateTotalChunks(request.FileSize, request.ChunkSize!.Value)
                    : null,
                StoragePath = _storage.GetTemporaryFilePath(uploadId),
                CreatedAt = now
            };

            _dbContext.Videos.Add(video);
            _dbContext.Uploads.Add(upload);

            await _dbContext.SaveChangesAsync(cancellationToken);

            try
            {
                if (request.Method == UploadMethod.Chunked)
                {
                    await _storage.CreateUploadFileAsync(
                        uploadId,
                        request.FileSize,
                        cancellationToken);
                }
            }
            catch (Exception)
            {
                upload.Status = UploadStatus.Failed;
                upload.FailedAt = DateTime.UtcNow;
                upload.ErrorMessage = "Failed to initialize upload storage";

                video.Status = VideoStatus.Failed;
                await _dbContext.SaveChangesAsync(cancellationToken);
                throw;
            }

            return new CreateUploadResponse
            {
                UploadId = upload.Id,
                VideoId = video.Id,
                Status = upload.Status,
                Method = upload.Method,
                TotalBytes = upload.TotalBytes,
                ChunkSize = upload.ChunkSize,
                TotalChunks = upload.TotalChunks,
            };
        }

        public async Task<bool> DeleteAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            using var uploadLock = await _uploadLockManager.AcquireAsync(
                uploadId,
                cancellationToken);

            var upload = await _dbContext.Uploads
                .Include(x => x.Video)
                .FirstOrDefaultAsync(x =>
                    x.Id == uploadId && x.OwnerId == userId,
                    cancellationToken);

            if (upload is null)
                return false;

            if (upload.Status == UploadStatus.Finalizing)
            {
                throw new InvalidOperationException(
                    "Upload is currently finalizing");
            }

            await _storage.DeleteUploadAsync(
                upload.Id,
                cancellationToken);

            _dbContext.Uploads.Remove(upload);
            _dbContext.Videos.Remove(upload.Video);

            await _dbContext.SaveChangesAsync(cancellationToken);

            return true;
        }

        public async Task<UploadStatusResponse?> GetStatusAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            var upload = await _dbContext.Uploads
                .AsNoTracking()
                .Include(x => x.Chunks)
                .FirstOrDefaultAsync(x =>
                    x.Id == uploadId && x.OwnerId == userId,
                    cancellationToken
                );

            if (upload is null)
                return null;

            return new UploadStatusResponse
            {
                UploadId = upload.Id,
                VideoId = upload.VideoId,
                Status = upload.Status,
                TotalBytes = upload.TotalBytes,
                UploadedBytes = upload.UploadedBytes,
                TotalChunks = upload.TotalChunks,
                UploadedChunks = upload.Chunks.Count,
                CreatedAt = upload.CreatedAt,
                StartedAt = upload.StartedAt,
                CompletedAt = upload.CompletedAt,
                ErrorMessage = upload.ErrorMessage,
            };
        }

        public async Task<UploadChunkResponse> UploadChunkAsync(
            string userId,
            Guid uploadId,
            int chunkNumber,
            Stream chunk,
            long contentLength,
            CancellationToken cancellationToken = default)
        {
            using var uploadLock =
                await _uploadLockManager.AcquireAsync(
                    uploadId,
                    chunkNumber,
                    cancellationToken);

            var upload = await GetUploadAsync(
                userId,
                uploadId,
                cancellationToken);

            EnsureUploadMethod(upload.Method, UploadMethod.Chunked);

            EnsureUploadAcceptingData(upload.Status);

            if (chunkNumber < 0 || chunkNumber >= upload.TotalChunks)
            {
                throw new ArgumentException("Invalid chunk number.");
            }

            if (contentLength <= 0)
                throw new ArgumentException("Chunk cannot be empty.");

            var expectedSize = GetExpectedChunkSize(
                upload,
                chunkNumber);

            if (contentLength != expectedSize)
            {
                throw new ArgumentException(
                    $"Invalid chunk size. Expected {expectedSize} bytes, received {contentLength} bytes.");
            }

            var existingChunk = await _dbContext.Chunks
                .FirstOrDefaultAsync(x =>
                    x.UploadId == uploadId &&
                    x.ChunkNumber == chunkNumber,
                    cancellationToken);

            if (existingChunk is not null)
                throw new InvalidOperationException("This chunk has already been uploaded.");

            var offset = checked((long)chunkNumber * upload.ChunkSize!.Value);

            await _storage.WriteChunkAsync(
                uploadId,
                chunk,
                offset,
                contentLength,
                cancellationToken);

            var uploadChunk = new UploadChunk
            {
                Id = Guid.NewGuid(),
                UploadId = uploadId,
                ChunkNumber = chunkNumber,
                Offset = offset,
                Size = checked((int)contentLength),
                UploadedAt = DateTime.UtcNow
            };

            _dbContext.Chunks.Add(uploadChunk);

            //upload.UploadedBytes += contentLength;

            await MarkUploadAsStatedAsync(upload, cancellationToken);

            await _dbContext.SaveChangesAsync(cancellationToken);

            //concurent requests could have changed the upload.UploadedBytes so, we get the current state
            var uploadedBytes = await _dbContext
                .Chunks
                .Where(x =>
                    x.UploadId == uploadId)
                .SumAsync(x => (long)x.Size,
                    cancellationToken);

            var uploadedChunks = await _dbContext
                .Chunks
                .CountAsync(x =>
                    x.UploadId == uploadId,
                    cancellationToken);

            return new UploadChunkResponse
            {
                UploadId = upload.Id,
                ChunkNumber = chunkNumber,
                Status = upload.Status,
                UploadedBytes = uploadedBytes,
                TotalBytes = upload.TotalBytes,
                UploadedChunks = uploadedChunks,
                TotalChunks = upload.TotalChunks!.Value
            };
        }

        public async Task<CompleteUploadResponse> UploadMultipartAsync(
            string userId,
            Guid uploadId,
            Stream file,
            long contentLength,
            CancellationToken cancellationToken = default)
        {
            using var uploadLock = await _uploadLockManager.AcquireAsync(
                uploadId,
                cancellationToken);

            var upload = await GetUploadAsync(
                userId,
                uploadId,
                cancellationToken,
                includeVideo: true);

            EnsureUploadMethod(upload.Method, UploadMethod.Multipart);

            EnsureUploadAcceptingData(upload.Status);

            ValidateContentLength(contentLength, upload.TotalBytes);

            await MarkUploadAsStatedAsync(upload, cancellationToken);

            /*
             * Streaming multipart file directly to tmp file
             */
            await _storage.WriteUploadAsync(
                uploadId,
                file,
                cancellationToken);

            await ValidateStoredFileSizeAsync(
                uploadId,
                upload.TotalBytes,
                cancellationToken);

            upload.UploadedBytes = upload.TotalBytes;

            return await FinalizeUploadAsync(upload, cancellationToken);
        }

        public async Task<CompleteUploadResponse> UploadStreamAsync(
            string userId,
            Guid uploadId,
            Stream file,
            long contentLength,
            CancellationToken cancellationToken = default)
        {
            using var uploadLock = await _uploadLockManager.AcquireAsync(
                uploadId,
                cancellationToken);

            var upload = await GetUploadAsync(
                userId,
                uploadId,
                cancellationToken,
                includeVideo: true);

            EnsureUploadMethod(upload.Method, UploadMethod.Stream);

            EnsureUploadAcceptingData(upload.Status);

            if (contentLength < 0)
            {
                throw new ArgumentException("Content-Length is required.");
            }

            ValidateContentLength(contentLength, upload.TotalBytes);

            await MarkUploadAsStatedAsync(upload, cancellationToken);

            /*
             * Streaming file directly to tmp file
             */
            await _storage.WriteUploadAsync(
                uploadId,
                file,
                cancellationToken);

            await ValidateStoredFileSizeAsync(
                uploadId,
                upload.TotalBytes,
                cancellationToken);

            upload.UploadedBytes = upload.TotalBytes;

            return await FinalizeUploadAsync(upload, cancellationToken);
        }

        public async Task<CompleteUploadResponse> CompleteAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            using var uploadLock =
                await _uploadLockManager.AcquireAsync(
                    uploadId,
                    cancellationToken);

            var upload = await GetUploadAsync(
                userId,
                uploadId,
                cancellationToken,
                includeVideo: true);

            /**
             * Idempotency - if multiple retries, return existing state
             * **/
            if (upload.Status == UploadStatus.Completed)
                return CompleteResponse(upload);

            EnsureUploadMethod(upload.Method, UploadMethod.Chunked);

            if (upload.Status is
                UploadStatus.Cancelled or
                UploadStatus.Failed)
            {
                throw new InvalidOperationException("This upload cannot be completed.");
            }

            if (upload.TotalChunks is null ||
                upload.ChunkSize is null)
            {
                throw new InvalidOperationException("Upload chunk configuration is invalid.");
            }

            var chunks = await _dbContext.Chunks
                .Where(x => x.UploadId == uploadId)
                .OrderBy(x => x.ChunkNumber)
                .ToListAsync(cancellationToken);

            if (chunks.Count != upload.TotalChunks.Value)
            {
                throw new InvalidOperationException("Not all chunks have been uploaded.");
            }

            // validate every chunk 
            for (var chunkNb = 0; chunkNb < chunks.Count; chunkNb++)
            {
                var chunk = chunks[chunkNb];
                if (chunk.ChunkNumber != chunkNb)
                {
                    throw new InvalidOperationException(
                        "Upload contains an invalid chunk sequence.");
                }
                var expectedOffset = (long)chunkNb * upload.ChunkSize.Value;

                if (chunk.Offset != expectedOffset)
                {
                    throw new InvalidOperationException(
                        "Upload contains an invalid chunk offset.");
                }

                var expectedSize = GetExpectedChunkSize(upload, chunkNb);
                if (chunk.Size != expectedSize)
                {
                    throw new InvalidOperationException(
                        "Upload contains an invalid chunk size.");
                }
            }


            var expectedBytes = chunks.Sum(x => (long)x.Size);

            upload.UploadedBytes = expectedBytes;

            if (expectedBytes != upload.TotalBytes)
            {
                throw new InvalidOperationException(
                    "Uploaded chunk sizes do not match the expected file size.");
            }

            //if (upload.UploadedBytes != upload.TotalBytes)
            //{
            //    throw new InvalidOperationException("Upload is not complete.");
            //}

            var actualFileSize = await _storage.GetFileSizeAsync(uploadId, cancellationToken);

            if (actualFileSize != upload.TotalBytes)
            {
                throw new InvalidOperationException(
                    "Temporary file size does not match the expected file size.");
            }

            return await FinalizeUploadAsync(upload, cancellationToken);
        }

        private async Task<CompleteUploadResponse> FinalizeUploadAsync(
            Upload upload,
            CancellationToken cancellationToken)
        {
            var extension = GetFileExtension(upload.Video.FileName);

            var finalFileExists =
                await _storage.VideoFileExistsAsync(
                    upload.VideoId,
                    extension,
                    cancellationToken);

            var temporaryFileExists =
                await _storage.UploadFileExistsAsync(
                    upload.Id,
                    cancellationToken);

            if (!finalFileExists)
            {
                if (!temporaryFileExists)
                {
                    throw new InvalidOperationException("Temporary upload file doesn't exist");
                }
                upload.Status = UploadStatus.Finalizing;
                upload.FinalizingAt = DateTime.UtcNow;

                await _dbContext.SaveChangesAsync(cancellationToken);

                await _storage.CompleteUploadAsync(
                    upload.Id,
                    upload.VideoId,
                    extension,
                    cancellationToken);
            }

            upload.Status = UploadStatus.Completed;
            upload.CompletedAt ??= DateTime.UtcNow;
            upload.Video.Status = VideoStatus.Processing;

            await _dbContext.SaveChangesAsync(cancellationToken);

            await _videoProcessingQueue.QueueAsync(
                upload.VideoId,
                cancellationToken);

            return CompleteResponse(upload);
        }

        private static CompleteUploadResponse CompleteResponse(Upload upload)
        {
            return new CompleteUploadResponse
            {
                UploadId = upload.Id,
                VideoId = upload.VideoId,
                Status = upload.Status,
                VideoStatus = upload.Video.Status,
                FileSize = upload.TotalBytes,
                CompletedAt = upload.CompletedAt!.Value
            };
        }
        private static int CalculateTotalChunks(
            long fileSize, int chunkSize)
        {
            return checked((int)((fileSize + chunkSize - 1) / chunkSize));
        }

        private static void EnsureUploadMethod(UploadMethod uploadMethod, UploadMethod expectedMethod)
        {
            if (uploadMethod != expectedMethod)
            {
                throw new InvalidOperationException(
                    $"This upload does not use {expectedMethod.ToString().ToLowerInvariant()} upload.");
            }
        }

        private static void EnsureUploadAcceptingData(UploadStatus uploadStatus)
        {
            if (uploadStatus is
                UploadStatus.Finalizing or
                UploadStatus.Completed or
                UploadStatus.Cancelled or
                UploadStatus.Failed)
            {
                throw new InvalidOperationException(
                    "This upload is no longer accepting data.");
            }
        }

        private async Task MarkUploadAsStatedAsync(
            Upload upload,
            CancellationToken cancellationToken)
        {
            if (upload.Status != UploadStatus.Created)
                return;

            upload.Status = UploadStatus.Uploading;
            upload.StartedAt = DateTime.UtcNow;
            await _dbContext.SaveChangesAsync(cancellationToken);
        }

        private static long GetExpectedChunkSize(Upload upload, int chunkNumber)
        {
            var chunkSize = upload.ChunkSize!.Value;

            var offset = (long)chunkNumber * chunkSize;

            var remaining = upload.TotalBytes - offset;

            return Math.Min(chunkSize, remaining);
        }

        private static string GetFileExtension(string fileName)
        {
            var extension = Path.GetExtension(fileName);
            if (string.IsNullOrWhiteSpace(extension))
                return ".bin";

            return extension.ToLowerInvariant();
        }

        private async Task<Upload> GetUploadAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken,
            bool includeVideo = false)
        {
            var query = _dbContext.Uploads.AsQueryable();

            if (includeVideo)
            {
                query = query.Include(x => x.Video);
            }

            var upload = await query
                .FirstOrDefaultAsync(x =>
                    x.Id == uploadId &&
                    x.OwnerId == userId,
                    cancellationToken);
            if (upload is null)
                throw new KeyNotFoundException("Upload was not found");
            return upload;
        }

        private static void ValidateContentLength(long expectedLength, long contentLength)
        {
            if (expectedLength != contentLength)
                throw new ArgumentException(
                    $"Invalid file size. Expected {expectedLength} bytes, received {contentLength} bytes.");
        }

        private async Task ValidateStoredFileSizeAsync(
            Guid uploadId,
            long expectedSize,
            CancellationToken cancellationToken)
        {
            var actualFileSize = await _storage.GetFileSizeAsync(
                uploadId,
                cancellationToken);
            if (actualFileSize != expectedSize)
            {
                throw new InvalidOperationException(
                    "Temporary file size does not match the expected file size.");
            }
        }

        private void ValidateRequest(CreateUploadRequest request)
        {
            if (string.IsNullOrWhiteSpace(request.Title))
                throw new ArgumentException("Title is required");

            if (request.Title.Length > 255)
                throw new ArgumentException(
                    "Title cannot exceed 255 characters.");

            if (request.Description?.Length > 5000)
                throw new ArgumentException(
                    "Description cannot exceed 5000 characters.");

            if (string.IsNullOrWhiteSpace(request.FileName))
                throw new ArgumentException(
                    "File name is required.");

            if (request.FileName.Length > 255)
                throw new ArgumentException(
                    "File name cannot exceed 255 characters.");

            if (request.FileSize <= 0)
                throw new ArgumentException(
                    "File size must be greater than zero.");

            if (request.FileSize > _uploadOptions.MaxFileSize)
                throw new ArgumentException(
                    $"File size cannot exceed {_uploadOptions.MaxFileSize} bytes.");

            if (!Enum.IsDefined(request.Method))
                throw new ArgumentException(
                    "Invalid upload method.");

            if (string.IsNullOrWhiteSpace(request.ContentType))
                throw new ArgumentException(
                    "Content type is required.");

            if (request.ContentType.Length > 100)
                throw new ArgumentException(
                    "Content type cannot exceed 100 characters.");

            if (request.Method == UploadMethod.Chunked)
            {
                if (request.ChunkSize is null)
                    throw new ArgumentException(
                        "Chunk size is required for chunked uploads.");

                if (request.ChunkSize < _uploadOptions.MinChunkSize)
                    throw new ArgumentException(
                        $"Chunk size must be at least {_uploadOptions.MinChunkSize} bytes.");

                if (request.ChunkSize > _uploadOptions.MaxChunkSize)
                    throw new ArgumentException(
                        $"Chunk size cannot exceed {_uploadOptions.MaxChunkSize} bytes.");
            }
            else if (request.ChunkSize is not null)
            {
                throw new ArgumentException(
                    "Chunk size can only be specified for chunked uploads.");
            }
        }
    }
}
