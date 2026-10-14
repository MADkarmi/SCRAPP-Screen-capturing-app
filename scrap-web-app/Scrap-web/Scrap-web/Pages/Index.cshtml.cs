using Microsoft.AspNetCore.Mvc.RazorPages;
using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;

namespace Scrap_web.Pages;

public class IndexModel : PageModel
{
    private readonly ILogger<IndexModel> _logger;
    private readonly ApplicationDbContext _dbContext;

    public IndexModel(
        ILogger<IndexModel> logger,
        ApplicationDbContext dbContext)
    {
        _logger = logger;
        _dbContext = dbContext;
    }

    public List<VideoCardViewModel> Videos { get; private set; } = [];

    public async Task OnGetAsync(CancellationToken cancellationToken)
    {

        Videos = await _dbContext.Videos
            .AsNoTracking()
            .Where(v =>
                v.Status == VideoStatus.Published)
            .OrderByDescending(v =>
                v.PublishedAt)
            .Select(v =>
            new VideoCardViewModel
            {
                VideoId = v.Id,
                Title = v.Title,
                UploaderName = v.Owner.UserName ?? "Unknown",
                UploadDate = v.PublishedAt ?? v.CreatedAt,
                ThumbnailUrl = string.Empty,//$"/thumbnails/{v.Id}.jpg",
                VideoUrl = string.Empty//$"/videos/{v.Id}"
            })
            .ToListAsync(cancellationToken);

    }
}
