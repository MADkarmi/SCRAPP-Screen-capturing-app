
using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;
using Scrap_web.Storage;

namespace Scrap_web.Services.VideoProcessing
{
    public sealed class BasicVideoProcessor : IVideoProcessor
    {
        private readonly ApplicationDbContext _dbContext;
        private readonly IVideoStorage _videoStorage;

        public BasicVideoProcessor(
            ApplicationDbContext dbContext,
            IVideoStorage videoStorage)
        {
            _dbContext = dbContext;
            _videoStorage = videoStorage;
        }

        public async Task ProcessVideoAsync(
            Guid videoId,
            CancellationToken cancellationToken = default)
        {
            var video = await _dbContext.Videos.
                FirstOrDefaultAsync(v =>
                v.Id == videoId,
                cancellationToken);

            if (video == null)
            {
                throw new ArgumentException($"Video with ID {videoId} not found.");
            }

            var extension = Path.GetExtension(video.FileName);

            if (!string.Equals(extension, ".mp4", StringComparison.OrdinalIgnoreCase))
            {
                throw new NotSupportedException($"Video format {extension} is not supported.");
            }

            var exists = await _videoStorage.
                VideoFileExistsAsync(
                    videoId,
                    extension,
                    cancellationToken);

            if (!exists)
            {
                throw new FileNotFoundException(
                    $"Video file for ID {videoId} not found in storage.");
            }

            /*
             * Here you would implement the actual video processing logic, 
             * such as transcoding, generating thumbnails, etc. 
             * This is a minimal implementation and does not include those details.
             */

            await Task.Delay(TimeSpan.FromSeconds(10), cancellationToken); // Simulate processing time

            video.ContentType = "video/mp4";
            video.Status = VideoStatus.Ready;

            await _dbContext.SaveChangesAsync(cancellationToken);

        }
    }
}
