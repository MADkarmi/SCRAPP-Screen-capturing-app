using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;
using Scrap_web.Storage;

namespace Scrap_web.Services.VideoService
{
    public class VideoService : IVideoService
    {
        private readonly ApplicationDbContext _dbContext;
        private readonly IVideoStorage _videoStorage;

        public VideoService(
            ApplicationDbContext dbContext,
            IVideoStorage videoStorage)
        {
            _dbContext = dbContext;
            _videoStorage = videoStorage;
        }

        public async Task<VideoOperationResult> PublishAsync(
            Guid videoId,
            string ownerId,
            CancellationToken cancellationToken = default)
        {
            var video = await _dbContext.Videos
                .FirstOrDefaultAsync(v =>
                    v.Id == videoId
                    && v.OwnerId == ownerId,
                    cancellationToken);

            if (video is null)
                return VideoOperationResult.NotFound;

            if (video.Status != VideoStatus.Ready)
                return VideoOperationResult.InvalidStatus;

            video.Status = VideoStatus.Published;
            video.PublishedAt = DateTime.UtcNow;

            await _dbContext.SaveChangesAsync(cancellationToken);

            return VideoOperationResult.Success;
        }

        public async Task<VideoOperationResult> DeleteAsync(
            Guid videoId,
            string ownerId,
            CancellationToken cancellationToken = default)
        {
            var video = await _dbContext.Videos
                .FirstOrDefaultAsync(v =>
                    v.Id == videoId
                    && v.OwnerId == ownerId,
                    cancellationToken);

            if (video is null)
                return VideoOperationResult.NotFound;

            _dbContext.Videos.Remove(video);
            await _dbContext.SaveChangesAsync(cancellationToken);

            await _videoStorage.DeleteVideoAsync(videoId, cancellationToken);

            return VideoOperationResult.Success;
        }
    }
}
