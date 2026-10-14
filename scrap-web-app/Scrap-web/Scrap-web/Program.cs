using Microsoft.AspNetCore.Http.Features;
using Microsoft.EntityFrameworkCore;
using Scrap_web.Configuration;
using Scrap_web.Data;
using Scrap_web.Services.Uploads;
using Scrap_web.Services.VideoProcessing;
using Scrap_web.Services.VideoService;
using Scrap_web.Storage;
using System.Text.Json;
using System.Text.Json.Serialization;


var builder = WebApplication.CreateBuilder(args);

builder.AddServiceDefaults();

// Add services to the container.
var connectionString = builder.Configuration.GetConnectionString("DefaultConnection") ?? throw new InvalidOperationException("Connection string 'DefaultConnection' not found.");
builder.Services.AddDbContext<ApplicationDbContext>(options =>
    options.UseSqlServer(connectionString));
builder.Services.AddDatabaseDeveloperPageExceptionFilter();

builder.Services.AddDefaultIdentity<ApplicationUser>(options => options.SignIn.RequireConfirmedAccount = true)
    .AddEntityFrameworkStores<ApplicationDbContext>();
builder.Services.AddOpenIddict()
    .AddCore(options =>
    {
        options.UseEntityFrameworkCore()
            .UseDbContext<ApplicationDbContext>();
    })
    .AddServer(options =>
    {
        options.SetAuthorizationEndpointUris("/connect/authorize");
        options.SetTokenEndpointUris("/connect/token");
        options.SetRevocationEndpointUris("/connect/revoke");
        options.SetLogoutEndpointUris("/connect/logout");
        options.AllowAuthorizationCodeFlow()
               .RequireProofKeyForCodeExchange()
               .AllowRefreshTokenFlow();
        //options.AllowPasswordFlow()
        //       .AllowRefreshTokenFlow();        
        options.AddDevelopmentEncryptionCertificate()
               .AddDevelopmentSigningCertificate();
        options.UseAspNetCore()
               .EnableAuthorizationEndpointPassthrough()
               .EnableLogoutEndpointPassthrough();
    })
    .AddValidation(options =>
    {
        options.UseLocalServer();
        options.UseAspNetCore();
    });

builder.Services.Configure<VideoStorageOptions>(
    builder.Configuration.GetSection("VideoStorage"));

builder.Services.Configure<VideoUploadConfiguration>(
    builder.Configuration.GetSection("VideoUpload"));

builder.Services.Configure<OpenIddictDesktopClientOptions>(
    builder.Configuration.GetSection(OpenIddictDesktopClientOptions.SectionName));

builder.Services.Configure<FormOptions>(options =>
{
    options.MultipartBodyLengthLimit = MultipartFormsLimit.MultipartFormMaxRequestBodySize;
    //options.BufferBodyLengthLimit = MultipartFormsLimit.MultipartFormMaxRequestBodySize;
});

builder.Services.AddSingleton<IVideoStorage, FileSystemVideoStorage>();

builder.Services.AddSingleton<IUploadLockManager, UploadLockManager>();

builder.Services.AddSingleton<IBackgroundVideoProcesingQueue, BackgroundVideoProcesingQueue>();

builder.Services.AddScoped<IVideoProcessor, BasicVideoProcessor>();

builder.Services.AddScoped<IUploadService, UploadService>();

builder.Services.AddScoped<IVideoService, VideoService>();

builder.Services.AddHostedService<BackgroundVideoProcessingService>();

builder.Services.AddControllers()
    .AddJsonOptions(options =>
    {
        options.JsonSerializerOptions.Converters.Add(
            new JsonStringEnumConverter());
        options.JsonSerializerOptions.PropertyNamingPolicy =
            JsonNamingPolicy.CamelCase;
    });

builder.Services.AddRouting(options =>
{
    options.LowercaseUrls = true;
});

builder.Services.AddRazorPages();

var app = builder.Build();

using (var scope = app.Services.CreateScope())
{
    var services = scope.ServiceProvider;
    //var context = services.GetRequiredService<ApplicationDbContext>();
    //context.Database.Migrate();
    await OpenIddictSeeder.SeedAsync(services);
}

app.MapDefaultEndpoints();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.UseMigrationsEndPoint();
}
else
{
    app.UseExceptionHandler("/Error");
    // The default HSTS value is 30 days. You may want to change this for production scenarios, see https://aka.ms/aspnetcore-hsts.
    app.UseHsts();
}

app.UseHttpsRedirection();
app.UseStaticFiles();

app.UseRouting();

app.UseAuthentication();
app.UseAuthorization();

app.MapControllers();
app.MapRazorPages();

app.Run();
