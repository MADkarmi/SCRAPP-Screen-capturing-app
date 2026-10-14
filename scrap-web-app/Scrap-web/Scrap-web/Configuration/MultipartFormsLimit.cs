namespace Scrap_web.Configuration
{
    public static class MultipartFormsLimit
    {
        /// <summary>
        /// The maximum size of the request body for multipart/form-data requests used in upload api.
        /// We set this limit to 10 GB to allow for large file uploads. 
        /// This limit is enforced by the server and can be adjusted based on the application's requirements.
        /// </summary>
        public const long MultipartFormMaxRequestBodySize = 10L * 1024 * 1024 * 1024;
    }
}
