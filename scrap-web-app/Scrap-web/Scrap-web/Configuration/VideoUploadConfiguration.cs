namespace Scrap_web.Configuration
{
    public sealed class VideoUploadConfiguration
    {
        public long MaxFileSize { get; set; } = 10L * 1024 * 1024 * 1024;
        public int MaxChunkSize { get; set; } = 100 * 1024 * 1024;
        public int MinChunkSize { get; set; } = 1 * 1024 * 1024;
    }
}
