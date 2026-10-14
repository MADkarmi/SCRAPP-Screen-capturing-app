using Microsoft.AspNetCore;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;

namespace Scrap_web.Controllers
{
    public class LogoutController : Controller
    {
        [HttpGet("~/connect/logout")]
        [HttpPost("~/connect/logout")]
        [IgnoreAntiforgeryToken]

        public async Task<IActionResult> Logout()
        {
            var request = HttpContext.GetOpenIddictServerRequest()
                ?? throw new InvalidOperationException(
                     "The OpenID Connect logout request cannot be retrieved.");
            await HttpContext.SignOutAsync(
                IdentityConstants.ApplicationScheme);

            if (!string.IsNullOrEmpty(request.PostLogoutRedirectUri))
            {
                return Redirect(request.PostLogoutRedirectUri);
            }

            return Redirect("~/");
        }
    }
}
