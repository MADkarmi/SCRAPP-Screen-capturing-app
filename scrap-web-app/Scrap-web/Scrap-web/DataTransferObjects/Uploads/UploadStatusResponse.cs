using Scrap_web.Models.Uploads;

namespace Scrap_web.DataTransferObjects.Uploads
{
    public sealed class UploadStatusResponse
    {
        public Guid UploadId { get; set; }
        public Guid VideoId { get; set; }
        public UploadStatus Status { get; set; }
        public long TotalBytes { get; set; }
        public long UploadedBytes { get; set; }
        public int? TotalChunks { get; set; }
        public int UploadedChunks { get; set; }
        public DateTime CreatedAt { get; set; }
        public DateTime? StartedAt { get; set; }
        public DateTime? CompletedAt { get; set; }
        public string? ErrorMessage { get; set; }
    }
}
