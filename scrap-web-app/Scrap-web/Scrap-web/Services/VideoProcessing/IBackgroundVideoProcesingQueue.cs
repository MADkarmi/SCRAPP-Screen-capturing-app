namespace Scrap_web.Services.VideoProcessing
{
    public interface IBackgroundVideoProcesingQueue
    {
        ValueTask QueueAsync(
            Guid videoId,
            CancellationToken cancellationToken = default);
        ValueTask<Guid> DequeueAsync(
            CancellationToken cancellationToken = default);
    }
}
