using Scrap_web.Models.Uploads;

namespace Scrap_web.DataTransferObjects.Uploads
{
    public sealed class UploadChunkResponse
    {
        public Guid UploadId { get; set; }
        public int ChunkNumber { get; set; }        
        public UploadStatus Status { get; set; }
        public long TotalBytes { get; set; }
        public long UploadedBytes { get; set; }
        public int? TotalChunks { get; set; }
        public int UploadedChunks { get; set; }
    }
}
