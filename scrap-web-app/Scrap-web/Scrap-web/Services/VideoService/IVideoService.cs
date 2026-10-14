namespace Scrap_web.Services.VideoService
{
    public interface IVideoService
    {
        Task<VideoOperationResult> DeleteAsync(
            Guid videoId,
            string ownerId,
            CancellationToken cancellationToken = default);

        Task<VideoOperationResult> PublishAsync(
            Guid videoId,
            string ownerId,
            CancellationToken cancellationToken = default);
    }
}
