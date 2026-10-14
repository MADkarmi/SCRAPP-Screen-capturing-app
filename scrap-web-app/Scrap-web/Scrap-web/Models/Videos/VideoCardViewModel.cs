namespace Scrap_web.Models.Videos
{
    public class VideoCardViewModel
    {
        public Guid VideoId { get; set; }
        public string Title { get; set; } = string.Empty;
        public string UploaderName { get; set; } = string.Empty;
        public DateTime UploadDate { get; set; } = DateTime.Now;
        public string ThumbnailUrl { get; set; } = string.Empty;
        public string VideoUrl { get; set; } = string.Empty;
    }
}