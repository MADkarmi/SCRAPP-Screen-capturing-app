using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;
using Microsoft.EntityFrameworkCore;
using Scrap_web.Data;
using Scrap_web.Models.Videos;

namespace Scrap_web.Pages.Videos
{
    public class WatchModel : PageModel
    {
        private readonly ApplicationDbContext _dbContext;
        private readonly UserManager<ApplicationUser> _userManager;

        public WatchModel(
            ApplicationDbContext dbContext, 
            UserManager<ApplicationUser> userManager)
        {
            _dbContext = dbContext;
            _userManager = userManager;
        }

        public Video? Video { get; private set; }

        public async Task<IActionResult> OnGetAsync(
            Guid id, 
            CancellationToken cancellationToken)
        {
            Video = await _dbContext.Videos
                .AsNoTracking()
                .Include(v => v.Owner)
                .FirstOrDefaultAsync(v => 
                    v.Id == id,
                    cancellationToken);

            if (Video == null)
                return NotFound();

            if(Video.Status == VideoStatus.Published)
                return Page();
            
            var userId = _userManager.GetUserId(User);
            
            if(Video.OwnerId != userId)
                return NotFound();

            return Page();
        }
    }
}