using Microsoft.AspNetCore.Http.Features;

namespace Scrap_web.Configuration
{
    public static class UploadRequestLimits
    {
        public static void Set(
            HttpContext context,
            long maxRequestBodySize)
        {
            var maxRequestBodySizeFeature = context.Features.Get<IHttpMaxRequestBodySizeFeature>();
            if (maxRequestBodySizeFeature is not null)
            {
                maxRequestBodySizeFeature.MaxRequestBodySize = maxRequestBodySize;
            }
        }
    }
}
