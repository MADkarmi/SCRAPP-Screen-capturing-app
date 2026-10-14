using System.Threading.Channels;

namespace Scrap_web.Services.VideoProcessing
{
    public class BackgroundVideoProcesingQueue : IBackgroundVideoProcesingQueue
    {
        private readonly Channel<Guid> _queue;

        public BackgroundVideoProcesingQueue(int capacity = 100)
        {
            _queue = Channel.CreateBounded<Guid>(
                new BoundedChannelOptions(capacity)
                {
                    FullMode = BoundedChannelFullMode.Wait,
                    SingleReader = true,
                    SingleWriter = false
                });
        }

        public ValueTask<Guid> DequeueAsync(
            CancellationToken cancellationToken = default)
        {
            return _queue.Reader.ReadAsync(cancellationToken);
        }

        public ValueTask QueueAsync(
            Guid videoId,
            CancellationToken cancellationToken = default)
        {
            return _queue.Writer.WriteAsync(videoId, cancellationToken);
        }
    }
}
