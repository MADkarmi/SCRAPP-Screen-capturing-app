using Scrap_web.Models.Uploads;
using Scrap_web.Models.Videos;

namespace Scrap_web.DataTransferObjects.Uploads
{
    public sealed class CompleteUploadResponse
    {
        public Guid UploadId { get; set; }
        public Guid VideoId { get; set; }
        public UploadStatus Status { get; set; }
        public VideoStatus VideoStatus { get; set; }
        public long FileSize { get; set; }
        public DateTime CompletedAt { get; set; }
    }
}
