Start tailwind watch process for development:

poweshell > Scrap-web\Scrap-web\ npx tailwindcss -i wwwroot/css/site.css -o wwwroot/css/site.min.css --watch

## System Preparation
# .NET 8 SDK is required

1. Change in **appsettings.json**
    ```
    "DefaultConnection": //"Server=YourServer;Database=YourDatabase;User Id=YourUser;Password=YourPassword;"

    "RedirectUri": "Your redirect URI",
    "PostLogoutRedirectUri": "Your post-logout redirect URI"
    ```
    in **Scrap-web.csproj**
    ```
    <UserSecretsId><!--Your User Secrets ID--></UserSecretsId>
    ```
    in **OpenIddictDesktopClientOptions**
    ```
    public string RedirectUri { get; set; } =
        "your application RedirectUri/callback";

    public string PostLogoutRedirectUri { get; set; } =
        "your application PostLogoutRedirectUri/logout-callback";
    ```
    in **launchSettings.json** lines with
    ```
    "applicationUrl": your application URL,
    ```

2. Make sure that EF Core is installed ``` dotnet tool list --global ```
3. Add trusted dev cert ```dotnet dev-certs https --trust```
4. ```dotnet restore```
5. ```dotnet buid```
6. ```dotnet-ef database update```
7. ```dotnet run --launch-profile https```