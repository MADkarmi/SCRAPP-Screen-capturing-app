
using Microsoft.Extensions.Options;

namespace Scrap_web.Storage
{
    public sealed class FileSystemVideoStorage : IVideoStorage
    {
        private readonly string _rootPath;
        private readonly int bufferSize = 1024 * 1024;

        public FileSystemVideoStorage(
            IOptions<VideoStorageOptions> options,
            IWebHostEnvironment environment)
        {
            var configurePath = options.Value.RoothPath;

            _rootPath = Path.IsPathRooted(configurePath)
                ? configurePath
                : Path.Combine(environment.ContentRootPath, configurePath);

            Directory.CreateDirectory(GetUploadsRoot());
            Directory.CreateDirectory(GetVideosRoot());
        }

        public Task CompleteUploadAsync(
            Guid uploadId,
            Guid videoId,
            string extension,
            CancellationToken cancellationToken = default)
        {
            var tmpPath = GetTemporaryFilePath(uploadId);
            var finalPath = GetVideoFilePath(videoId, extension);

            if (!File.Exists(tmpPath))
            {
                throw new FileNotFoundException(
                    "Temporary upload file was not found.",
                    tmpPath);
            }

            var directory = Path.GetDirectoryName(finalPath);
            if (directory is not null)
                Directory.CreateDirectory(GetVideosRoot());

            if (File.Exists(finalPath))
            {
                if (File.Exists(tmpPath))
                {
                    File.Delete(tmpPath);
                }
                return Task.CompletedTask;
            }

            if (!File.Exists(tmpPath))
            {
                throw new FileNotFoundException(
                    "Temporary upload file was not found.",
                    tmpPath);
            }
            File.Move(tmpPath, finalPath, overwrite: false);

            var uploadDirectory = GetUploadDirectory(uploadId);
            if (Directory.Exists(uploadDirectory))
            {
                Directory.Delete(uploadDirectory, true);
            }
            return Task.CompletedTask;
        }

        public Task<Stream> CreateUploadStreamAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            var directory = GetUploadDirectory(uploadId);
            Directory.CreateDirectory(directory);

            var path = GetTemporaryFilePath(uploadId);

            return Task.FromResult<Stream>(
                new FileStream(
                    path,
                    FileMode.Create,
                    FileAccess.Write,
                    FileShare.None,
                    bufferSize: bufferSize,
                    options: FileOptions.Asynchronous | FileOptions.SequentialScan));
        }

        public Task DeleteUploadAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            var directory = GetUploadDirectory(uploadId);

            if (Directory.Exists(directory))
            {
                Directory.Delete(directory, true);
            }
            return Task.CompletedTask;
        }

        public Task DeleteVideoAsync(
            Guid videoId,
            CancellationToken cancellationToken = default)
        {
            var videoPath = GetVideoFilePath(videoId);
            if (File.Exists(videoPath))
            {
                File.Delete(videoPath);
            }
            return Task.CompletedTask;
        }

        public string GetTemporaryFilePath(Guid uploadId)
        {
            return Path.Combine(GetUploadsRoot(), uploadId.ToString("N"), "upload.tmp");
        }

        public string GetUploadDirectory(Guid uploadId)
        {
            return Path.Combine(GetUploadsRoot(), uploadId.ToString("N"));
        }

        public string GetVideoFilePath(Guid videoId)
        {
            return Path.Combine(GetVideosRoot(), $"{videoId:N}.mp4");
        }

        public string GetVideoFilePath(Guid videoId, string extension)
        {
            extension = NormalizeExtension(extension);
            return Path.Combine(GetVideosRoot(), $"{videoId:N}{extension}");
        }

        public async Task CreateUploadFileAsync(Guid uploadId, long fileSize, CancellationToken cancellationToken = default)
        {
            var directory = GetUploadDirectory(uploadId);
            Directory.CreateDirectory(directory);
            var path = GetTemporaryFilePath(uploadId);

            await using var stream = new FileStream(
                path,
                FileMode.Create,
                FileAccess.Write,
                FileShare.Read,
                bufferSize: bufferSize,// 1024 * 1024,
                options: FileOptions.Asynchronous | FileOptions.SequentialScan);

            stream.SetLength(fileSize);
        }

        public async Task WriteChunkAsync(
            Guid uploadId,
            Stream chunk,
            long offset,
            long length,
            CancellationToken cancellationToken = default)
        {
            var path = GetTemporaryFilePath(uploadId);
            if (!File.Exists(path))
            {
                throw new FileNotFoundException(
                    "Temporary upload file was not found",
                    path);
            }
            //var buffersize = 1024 * 1024;

            await using var fileStream = new FileStream(
                path,
                FileMode.Open,
                FileAccess.Write,
                FileShare.ReadWrite,
                bufferSize: bufferSize,
                options: FileOptions.Asynchronous | FileOptions.RandomAccess);

            var buffer = new byte[bufferSize];

            long position = offset;
            long remaining = length;

            while (remaining > 0)
            {
                var bytesToRead = (int)Math.Min(remaining, buffer.Length);

                var bytesRead = await chunk.ReadAsync(
                    buffer.AsMemory(0, bytesToRead),
                    cancellationToken);

                if (bytesRead == 0)
                {
                    throw new EndOfStreamException("Chunk ended before the expected number of bytes was received.");
                }

                await RandomAccess.WriteAsync(
                    fileStream.SafeFileHandle,
                    buffer.AsMemory(0, bytesRead),
                    position,
                    cancellationToken);

                position += bytesRead;
                remaining -= bytesRead;
            }
        }

        public async Task WriteUploadAsync(
            Guid uploadId,
            Stream source,
            CancellationToken cancellationToken = default)
        {
            var path = GetTemporaryFilePath(uploadId);
            var directory = Path.GetDirectoryName(path);
            if (directory is not null)
                Directory.CreateDirectory(directory);

            await using var output = new FileStream(
                path,
                FileMode.Create,
                FileAccess.Write,
                FileShare.None,
                bufferSize: bufferSize,
                options: FileOptions.Asynchronous | FileOptions.SequentialScan);

            // don't load data into memory
            await source.CopyToAsync(output, cancellationToken);
            await output.FlushAsync(cancellationToken);

        }

        public Task<long> GetFileSizeAsync(Guid uploadId, CancellationToken cancellation = default)
        {
            var path = GetTemporaryFilePath(uploadId);
            if (!File.Exists(path))
            {
                throw new FileNotFoundException(
                    "Temporary upload file was not found",
                    path);
            }

            var info = new FileInfo(path);
            return Task.FromResult(info.Length);
        }

        public Task<bool> UploadFileExistsAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            return Task.FromResult(File.Exists(GetTemporaryFilePath(uploadId)));
        }

        public Task<bool> VideoFileExistsAsync(
            Guid videoId,
            string extension,
            CancellationToken cancellationToken = default)
        {
            return Task.FromResult(File.Exists(GetVideoFilePath(videoId, extension)));
        }

        private string GetVideosRoot()
        {
            return Path.Combine(_rootPath, "videos");
        }

        private string GetUploadsRoot()
        {
            return Path.Combine(_rootPath, "uploads");
        }

        private static string NormalizeExtension(string extension)
        {
            if (string.IsNullOrWhiteSpace(extension))
                return ".bin";
            return extension.StartsWith('.')
                ? extension.ToLowerInvariant()
                : "." + extension.ToLowerInvariant();
        }
    }
}
