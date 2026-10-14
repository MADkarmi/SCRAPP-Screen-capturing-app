using Scrap_web.Models.Uploads;

namespace Scrap_web.DataTransferObjects.Uploads
{
    public sealed class CreateUploadResponse
    {
        public Guid UploadId { get; set; }
        public Guid VideoId { get; set; }
        public UploadStatus Status { get; set; }
        public UploadMethod Method { get; set; }
        public long TotalBytes { get; set; }
        public int? ChunkSize { get; set; }
        public int? TotalChunks { get; set; }
    }
}
