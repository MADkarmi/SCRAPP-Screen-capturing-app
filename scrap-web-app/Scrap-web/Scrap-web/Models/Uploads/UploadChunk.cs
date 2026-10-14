namespace Scrap_web.Models.Uploads
{
    public class UploadChunk
    {
        public Guid Id { get; set; }
        public Guid UploadId { get; set; }
        public Upload Upload { get; set; } = null!;
        public int ChunkNumber { get; set; }
        public long Offset { get; set; }
        public int Size { get; set; }
        public DateTime UploadedAt { get; set; }
    }
}