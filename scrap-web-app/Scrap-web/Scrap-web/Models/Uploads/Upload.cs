using Scrap_web.Data;
using Scrap_web.Models.Videos;

namespace Scrap_web.Models.Uploads
{
    public class Upload
    {
        public Guid Id { get; set; }
        public Guid VideoId { get; set; }
        public Video Video { get; set; } = null!;
        public string OwnerId { get; set; } = null!;
        public ApplicationUser Owner { get; set; } = null!;
        public UploadMethod Method { get; set; }
        public UploadStatus Status { get; set; }
        public long TotalBytes { get; set; }
        public long UploadedBytes { get; set; }
        public int? ChunkSize { get; set; }
        public int? TotalChunks { get; set; }
        public string StoragePath { get; set; } = null!;
        public DateTime CreatedAt { get; set; }
        public DateTime? StartedAt { get; set; }
        public DateTime? CompletedAt { get; set; }
        public DateTime? FailedAt { get; set; }
        
        public DateTime? FinalizingAt { get; set; }

        public string? ErrorMessage { get; set; }
        public ICollection<UploadChunk> Chunks { get; set; } = new List<UploadChunk>();
    }
}
