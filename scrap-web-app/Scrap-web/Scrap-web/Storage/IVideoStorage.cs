namespace Scrap_web.Storage
{
    public interface IVideoStorage
    {
        string GetUploadDirectory(Guid uploadId);
        string GetTemporaryFilePath(Guid uploadId);
        string GetVideoFilePath(Guid videoId, string extension);

        string GetVideoFilePath(Guid videoId);

        Task CreateUploadFileAsync(
            Guid uploadId,
            long fileSize,
            CancellationToken cancellationToken = default);

        Task<Stream> CreateUploadStreamAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default);

        Task WriteChunkAsync(
            Guid uploadId,
            Stream chunk,
            long offset,
            long length,
            CancellationToken cancellationToken = default);

        Task DeleteUploadAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default);

        Task CompleteUploadAsync(
            Guid uploadId,
            Guid videoId,
            string extension,
            CancellationToken cancellationToken = default);

        Task<long> GetFileSizeAsync(
            Guid uploadId,
            CancellationToken cancellation = default);

        Task<bool> UploadFileExistsAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default);

        Task<bool> VideoFileExistsAsync(
            Guid videoId,
            string extension,
            CancellationToken cancellationToken = default);

        Task WriteUploadAsync(
            Guid uploadId,
            Stream source,
            CancellationToken cancellationToken = default);

        Task DeleteVideoAsync(
            Guid videoId,
            CancellationToken cancellationToken = default);
    }
}
