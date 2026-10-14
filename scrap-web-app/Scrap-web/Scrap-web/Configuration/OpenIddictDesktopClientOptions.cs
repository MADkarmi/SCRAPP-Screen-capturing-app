using static System.Net.WebRequestMethods;

namespace Scrap_web.Configuration
{
    public class OpenIddictDesktopClientOptions
    {
        public const string SectionName = "OpenIddict:DesktopClient";
        public string ClientId { get; set; } = "scrap-desktop-app";

        public string DisplayName { get; set; } = "Scrap Desktop App";

        public string RedirectUri { get; set; } = "your application RedirectUri/callback";

        public string PostLogoutRedirectUri { get; set; } = "your application PostLogoutRedirectUri/logout-callback";
    }
}
