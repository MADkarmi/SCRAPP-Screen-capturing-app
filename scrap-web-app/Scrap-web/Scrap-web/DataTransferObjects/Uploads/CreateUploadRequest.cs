using Scrap_web.Models.Uploads;

namespace Scrap_web.DataTransferObjects.Uploads
{
    public sealed class CreateUploadRequest
    {
        public string Title { get; set; } = null!;
        public string? Description { get; set; }

        public string FileName { get; set; } = null!;
        public string ContentType { get; set; } = null!;
        public long FileSize { get; set; }
        public UploadMethod Method { get; set; }
        public int? ChunkSize { get; set; }
    }
}
