namespace Scrap_web.Services.Uploads
{
    public interface IUploadLockManager
    {
        Task<IDisposable> AcquireAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default);

        Task<IDisposable> AcquireAsync(
            Guid uploadId,
            int chunkNumber,
            CancellationToken cancellationToken = default);
    }
}
