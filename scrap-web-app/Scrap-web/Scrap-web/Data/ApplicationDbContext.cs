using Microsoft.AspNetCore.Identity.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore;
using Scrap_web.Models.Uploads;
using Scrap_web.Models.Videos;

namespace Scrap_web.Data;

public class ApplicationDbContext : IdentityDbContext<ApplicationUser>
{
    public ApplicationDbContext(DbContextOptions<ApplicationDbContext> options)
        : base(options)
    {
    }

    public DbSet<Video> Videos => Set<Video>();
    public DbSet<Upload> Uploads => Set<Upload>();
    public DbSet<UploadChunk> Chunks => Set<UploadChunk>();

    protected override void OnModelCreating(ModelBuilder builder)
    {
        base.OnModelCreating(builder);
        builder.UseOpenIddict();

        builder.Entity<Video>(entity =>
        {
            entity.HasKey(x => x.Id);

            entity.Property(x => x.Title)
                .HasMaxLength(255)
                .IsRequired();

            entity.Property(x => x.Description)
                .HasMaxLength(5000);

            entity.Property(x => x.FileName)
                .HasMaxLength(255)
                .IsRequired();

            entity.Property(x => x.ContentType)
                .HasMaxLength(100)
                .IsRequired();

            entity.HasOne(x => x.Owner)
                .WithMany()
                .HasForeignKey(x => x.OwnerId)
                .OnDelete(DeleteBehavior.Restrict);
        });

        builder.Entity<Upload>(entity =>
        {
            entity.HasKey(x => x.Id);

            entity.Property(x => x.StoragePath)
                .HasMaxLength(1000)
                .IsRequired();

            entity.HasOne(x => x.Video)
                .WithMany(x => x.Uploads)
                .HasForeignKey(x => x.VideoId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasOne(x => x.Owner)
                .WithMany()
                .HasForeignKey(x => x.OwnerId)
                .OnDelete(DeleteBehavior.Restrict);

            entity.HasIndex(x => new
            {
                x.OwnerId,
                x.Status
            });
        });

        builder.Entity<UploadChunk>(entity =>
        {
            entity.HasKey(x => x.Id);

            entity.HasOne(x => x.Upload)
                .WithMany(x => x.Chunks)
                .HasForeignKey(x => x.UploadId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasIndex(x => new
            {
                x.UploadId,
                x.ChunkNumber
            })
            .IsUnique();
        });
    }


}
