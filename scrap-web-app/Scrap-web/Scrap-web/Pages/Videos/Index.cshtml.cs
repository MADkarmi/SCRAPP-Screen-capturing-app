using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;
using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;
using Scrap_web.Services.VideoService;

namespace Scrap_web.Pages.Videos
{
    [Authorize]
    public class IndexModel : PageModel
    {
        private readonly ApplicationDbContext _dbContext;
        private readonly UserManager<ApplicationUser> _userManager;
        private readonly IVideoService _videoService;

        [TempData]
        public string StatusMessage { get; set; } = "Video published successfully.";

        public IndexModel(
            ApplicationDbContext dbContext,
            UserManager<ApplicationUser> userManager,
            IVideoService videoService)
        {
            _dbContext = dbContext;
            _userManager = userManager;
            _videoService = videoService;
        }

        public IList<Video> Videos { get; private set; } = new List<Video>();

        public async Task OnGetAsync()
        {
            var userId = _userManager.GetUserId(User);

            Videos = await _dbContext.Videos
                .Where(v =>
                    v.OwnerId == userId)
                .OrderByDescending(v => v.CreatedAt)
                .ToListAsync();
        }

        public async Task<IActionResult> OnPostPublishAsync(
            Guid id,
            CancellationToken cancellationToken)
        {
            var userId = _userManager.GetUserId(User);

            if (userId is null)
                return Challenge();

            var result = await _videoService.PublishAsync(
                id,
                userId,
                cancellationToken);

            switch (result)
            {
                case VideoOperationResult.Success:
                    StatusMessage = "Video published successfully";
                    return RedirectToPage();
                case VideoOperationResult.NotFound:
                    return NotFound();
                case VideoOperationResult.InvalidStatus:
                    return BadRequest();
                default:
                    return StatusCode(StatusCodes.Status500InternalServerError);
            }
        }

        public async Task<IActionResult> OnPostDeleteAsync(
            Guid id,
            CancellationToken cancellationToken)
        {
            var userId = _userManager.GetUserId(User);
            if (userId is null)
                return Challenge();

            var result = await _videoService.DeleteAsync(
                id,
                userId,
                cancellationToken);

            switch (result)
            {
                case VideoOperationResult.Success:
                    StatusMessage = "Video deleted successfully.";
                    return RedirectToPage();

                case VideoOperationResult.NotFound:
                    return NotFound();

                default:
                    return StatusCode(StatusCodes.Status500InternalServerError);
            }
        }
    }
}
