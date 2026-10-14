
using Scrap_web.Data;
using Scrap_web.Models.Uploads;

namespace Scrap_web.Models.Videos
{
    public class Video
    {
        public Guid Id { get; set; }
        public string Title { get; set; } = null!;
        public string? Description { get; set; }
        public string FileName { get; set; } = null!;
        public string ContentType { get; set; } = null!;
        public long FileSize { get; set; }
        public TimeSpan? Duration { get; set; }
        public int? Width { get; set; }
        public int? Height { get; set; }
        public VideoStatus Status { get; set; }
        public DateTime CreatedAt { get; set; }
        public DateTime? PublishedAt { get; set; }
        public string OwnerId { get; set; } = null!;
        public ApplicationUser Owner { get; set; } = null!;
        public ICollection<Upload> Uploads { get; set; } = new List<Upload>();


    }
}
