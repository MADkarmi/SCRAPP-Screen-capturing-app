using Microsoft.AspNetCore;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.IdentityModel.Tokens;
using OpenIddict.Abstractions;
using OpenIddict.Server.AspNetCore;
using Scrap_web.Data;
using System.Security.Claims;

namespace Scrap_web.Controllers
{
    public class AuthorizationController : Controller
    {
        private readonly UserManager<ApplicationUser> _userManager;
        public AuthorizationController(UserManager<ApplicationUser> userManager)
        {
            _userManager = userManager;
        }

        [HttpGet("~/connect/authorize")]
        [HttpPost("~/connect/authorize")]
        [IgnoreAntiforgeryToken]
        public async Task<IActionResult> Authorize()
        {
            var request = HttpContext.GetOpenIddictServerRequest()
                ?? throw new InvalidOperationException("The OpenID Connect request cannot be retrieved.");

            // Check if the user is authenticated. If not, redirect to the login page.
            var userAuthenticated = await HttpContext.AuthenticateAsync(IdentityConstants.ApplicationScheme);

            if (!userAuthenticated.Succeeded)
            {
                // Redirect to the login page with the return URL set to the current request.
                return Challenge(
                    new AuthenticationProperties
                    {
                        RedirectUri = Request.PathBase + Request.Path + Request.QueryString
                    },
                    IdentityConstants.ApplicationScheme
                );
            }

            var user = await _userManager.GetUserAsync(userAuthenticated.Principal);
            if (user == null)
            {
                return Forbid(OpenIddictServerAspNetCoreDefaults.AuthenticationScheme);
            }

            //create identity claims to issue authorization code/token
            var identity = new ClaimsIdentity(
                TokenValidationParameters.DefaultAuthenticationType,
                OpenIddictConstants.Claims.Name,
                OpenIddictConstants.Claims.Role
            );

            var userId = await _userManager.GetUserIdAsync(user);
            var userName = await _userManager.GetUserNameAsync(user);
            var email = await _userManager.GetEmailAsync(user);

            identity.AddClaim(
                OpenIddictConstants.Claims.Subject,
                userId
            );

            if (!string.IsNullOrEmpty(userName))
            {
                identity.AddClaim(
                    OpenIddictConstants.Claims.Name,
                    userName
                );
            }

            if (!string.IsNullOrEmpty(email))
            {
                identity.AddClaim(
                    OpenIddictConstants.Claims.Email,
                    email
                );
            }

            var principal = new ClaimsPrincipal(identity);
            principal.SetScopes(request.GetScopes());
            //principal.SetResources("resource_server"); //change this to your resource server name)

            principal.SetAccessTokenLifetime(TimeSpan.FromMinutes(30));
            principal.SetRefreshTokenLifetime(TimeSpan.FromDays(30));

            identity.SetDestinations(claim =>
            {
                switch (claim.Type)
                {
                    case OpenIddictConstants.Claims.Name:
                        return new[] {
                            OpenIddictConstants.Destinations.AccessToken,
                            OpenIddictConstants.Destinations.IdentityToken
                        };
                    case OpenIddictConstants.Claims.Email:
                        return new[] {
                            OpenIddictConstants.Destinations.AccessToken,
                            OpenIddictConstants.Destinations.IdentityToken
                        };
                    default:
                        return new[] {
                            OpenIddictConstants.Destinations.AccessToken
                        };
                }
            });

            return SignIn(
                principal,
                OpenIddictServerAspNetCoreDefaults.AuthenticationScheme
            );
        }
    }
}
