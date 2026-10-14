namespace Scrap_web.Services.VideoProcessing
{
    public interface IVideoProcessor
    {
        Task ProcessVideoAsync(
            Guid videoId,
            CancellationToken cancellationToken = default);
    }
}
