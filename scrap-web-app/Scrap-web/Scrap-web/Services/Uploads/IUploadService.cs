using Scrap_web.DataTransferObjects.Uploads;

namespace Scrap_web.Services.Uploads
{
    public interface IUploadService
    {
        Task<CreateUploadResponse> CreateUploadAsync(
            string userId,
            CreateUploadRequest request,
            CancellationToken cancellationToken = default);

        Task<UploadChunkResponse> UploadChunkAsync(
            string userId,
            Guid uploadId,
            int chunkNumber,
            Stream chunk,
            long contentLength,
            CancellationToken cancellationToken = default);

        Task<CompleteUploadResponse> UploadMultipartAsync(
            string userId,
            Guid uploadId,
            Stream file,
            long contentLength,
            CancellationToken cancellationToken = default);

        Task<CompleteUploadResponse> UploadStreamAsync(
            string userId,
            Guid uploadId,
            Stream file,
            long contentLength,
            CancellationToken cancellationToken = default);

        Task<UploadStatusResponse?> GetStatusAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken = default);

        Task<CompleteUploadResponse> CompleteAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken = default);

        Task<bool> DeleteAsync(
            string userId,
            Guid uploadId,
            CancellationToken cancellationToken = default);


    }
}
