using Microsoft.Extensions.Options;
using OpenIddict.Abstractions;
using Scrap_web.Configuration;

namespace Scrap_web.Data
{
    public class OpenIddictSeeder
    {
        public static async Task SeedAsync(IServiceProvider serviceProvider)
        {
            var scopeManager = serviceProvider.GetRequiredService<IOpenIddictScopeManager>();

            var applicationManager = serviceProvider.GetRequiredService<IOpenIddictApplicationManager>();

            var options = serviceProvider.GetRequiredService<IOptions<OpenIddictDesktopClientOptions>>().Value;

            if (await scopeManager.FindByNameAsync("email") is null)
            {
                await scopeManager.CreateAsync(
                    new OpenIddictScopeDescriptor
                    {
                        Name = "email",
                        DisplayName = "Email access",
                    }
                );
            }

            if (await scopeManager.FindByNameAsync("profile") is null)
            {
                await scopeManager.CreateAsync(
                    new OpenIddictScopeDescriptor
                    {
                        Name = "profile",
                        DisplayName = "User profile",
                    }
                );
            }

            var descriptor = new OpenIddictApplicationDescriptor
            {
                ClientId = options.ClientId,
                DisplayName = options.DisplayName,
                ClientType = OpenIddictConstants.ClientTypes.Public,
                ConsentType = OpenIddictConstants.ConsentTypes.Implicit, //change this to explicit if you want to show consent screen
                Permissions =
                {
                    OpenIddictConstants.Permissions.Endpoints.Authorization,
                    OpenIddictConstants.Permissions.Endpoints.Token,
                    OpenIddictConstants.Permissions.Endpoints.Logout,
                    OpenIddictConstants.Permissions.Endpoints.Revocation,
                    OpenIddictConstants.Permissions.GrantTypes.AuthorizationCode,
                    OpenIddictConstants.Permissions.GrantTypes.RefreshToken,
                    OpenIddictConstants.Permissions.ResponseTypes.Code,
                    OpenIddictConstants.Permissions.Scopes.Email,
                    OpenIddictConstants.Permissions.Scopes.Profile


                },
                RedirectUris =
                {
                    //change this to dynamically allocated port for desktop app or use a custom URI scheme
                    new Uri(options.RedirectUri) //change to auth-callback
                },
                PostLogoutRedirectUris =
                {
                    new Uri(options.PostLogoutRedirectUri)
                },
                Requirements =
                {
                    OpenIddictConstants.Requirements.Features.ProofKeyForCodeExchange
                }
            };

            var application = await applicationManager.FindByClientIdAsync(options.ClientId);

            if (application is null)
            {
                await applicationManager.CreateAsync(descriptor);
            }
            else
            {
                await applicationManager.UpdateAsync(application, descriptor);
            }

        }
    }
}
