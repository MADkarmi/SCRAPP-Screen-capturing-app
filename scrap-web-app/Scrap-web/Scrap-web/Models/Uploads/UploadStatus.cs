namespace Scrap_web.Models.Uploads
{
    public enum UploadStatus
    {
        Created = 0,
        Uploading = 1,
        Completed = 2,
        Failed = 3,
        Cancelled = 4,
        Finalizing = 5,
    }
}
