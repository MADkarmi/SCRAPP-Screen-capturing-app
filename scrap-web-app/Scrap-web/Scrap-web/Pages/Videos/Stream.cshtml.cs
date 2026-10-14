using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;
using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;
using Scrap_web.Storage;

namespace Scrap_web.Pages.Videos
{
    public class StreamModel : PageModel
    {
        private readonly ApplicationDbContext _context;
        private readonly UserManager<ApplicationUser> _userManager;
        private readonly IVideoStorage _videoStorage;

        public StreamModel(
            ApplicationDbContext context,
            UserManager<ApplicationUser> userManager,
            IVideoStorage videoStorage)
        {
            _context = context;
            _userManager = userManager;
            _videoStorage = videoStorage;
        }

        public async Task<IActionResult> OnGetAsync(
            Guid id,
            CancellationToken cancellationToken)
        {
            var video = await _context.Videos
                .AsNoTracking()
                .FirstOrDefaultAsync(v =>
                    v.Id == id,
                    cancellationToken);

            if (video is null)
                return NotFound();

            if (video.Status != VideoStatus.Published)
            {
                var user = await _userManager.GetUserAsync(User);
                if (user is null || video.OwnerId != user.Id)
                    return NotFound();
            }

            var filePath = _videoStorage.GetVideoFilePath(video.Id);

            if (!System.IO.File.Exists(filePath))
                return NotFound();

            var result = new PhysicalFileResult(
                filePath,
                video.ContentType)
            {
                EnableRangeProcessing = true
            };

            return result;
        }
    }
}
