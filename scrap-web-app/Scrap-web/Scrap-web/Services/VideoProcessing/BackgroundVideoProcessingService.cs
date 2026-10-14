using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;

namespace Scrap_web.Services.VideoProcessing
{
    public sealed class BackgroundVideoProcessingService : BackgroundService
    {
        private readonly IServiceScopeFactory _scopeFactory;
        private readonly IBackgroundVideoProcesingQueue _queue;
        private readonly ILogger<BackgroundVideoProcessingService> _logger;

        public BackgroundVideoProcessingService(
            IServiceScopeFactory scopeFactory,
            IBackgroundVideoProcesingQueue queue,
            ILogger<BackgroundVideoProcessingService> logger)
        {
            _scopeFactory = scopeFactory;
            _queue = queue;
            _logger = logger;
        }

        protected override async Task ExecuteAsync(CancellationToken stoppingToken)
        {
            _logger.LogInformation("Background video processing service started");


            while (!stoppingToken.IsCancellationRequested)
            {
                try
                {
                    var videoId = await _queue.DequeueAsync(stoppingToken);

                    await ProcessVideoAsync(videoId, stoppingToken);
                }
                catch (OperationCanceledException) 
                    when (stoppingToken.IsCancellationRequested)
                {
                    // Graceful shutdown
                    break;
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Error while processing video");
                }
            }
            _logger.LogInformation("Background video processing service stopped");
        }

        private async Task ProcessVideoAsync(Guid videoId, CancellationToken cancellationToken)
        {
            using var scope = _scopeFactory.CreateScope();
            var videoProcessingService = scope.ServiceProvider.GetRequiredService<IVideoProcessor>();
            //await videoProcessingService.ProcessVideoAsync(videoId, cancellationToken);

            try
            {
                _logger.LogInformation(
                    "Processing video with ID: {VideoId}",
                    videoId);

                await videoProcessingService
                    .ProcessVideoAsync(videoId, cancellationToken);

                _logger.LogInformation(
                    "Successfully processed video with ID: {VideoId}",
                    videoId);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, 
                    "Error while processing video with ID: {VideoId}", 
                    videoId);

                var dbContext = scope.ServiceProvider.GetRequiredService<ApplicationDbContext>();

                var video = await dbContext
                    .Videos.FirstOrDefaultAsync(v => 
                        v.Id == videoId,
                        cancellationToken);

                if (video is not null) 
                { 
                    video.Status = VideoStatus.Failed;
                    await dbContext.SaveChangesAsync(cancellationToken);
                }
            }
        }
    }
}
