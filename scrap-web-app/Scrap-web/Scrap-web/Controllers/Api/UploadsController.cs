using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Http.Features;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Options;
using OpenIddict.Abstractions;
using OpenIddict.Validation.AspNetCore;
using Scrap_web.Configuration;
using Scrap_web.DataTransferObjects.Uploads;
using Scrap_web.Services.Uploads;
using System.Security.Claims;

namespace Scrap_web.Controllers.Api
{
    [ApiController]
    [Route("api/uploads")]
    [Authorize(AuthenticationSchemes = OpenIddictValidationAspNetCoreDefaults.AuthenticationScheme)]
    public class UploadsController : ControllerBase
    {
        private readonly IUploadService _uploadService;
        private readonly VideoUploadConfiguration _uploadConfiguration;

        private string userId =>
            User.FindFirstValue(OpenIddictConstants.Claims.Subject)
            ?? throw new UnauthorizedAccessException();

        public UploadsController(
            IUploadService uploadService,
            IOptions<VideoUploadConfiguration> uploadConfiguration)
        {
            _uploadService = uploadService;
            _uploadConfiguration = uploadConfiguration.Value;
        }

        [HttpPost]
        public async Task<ActionResult<CreateUploadResponse>> Create(
            [FromBody] CreateUploadRequest request,
            CancellationToken cancellationToken)
        {
            try
            {
                var result = await _uploadService.CreateUploadAsync(
                    userId,
                    request,
                    cancellationToken);

                return Ok(result);
            }
            catch (ArgumentException ex)
            {
                return BadRequest(new { error = ex.Message });
            }
        }

        [HttpGet("{id:guid}")]
        public async Task<ActionResult<UploadStatusResponse>> Get(
            Guid id,
            CancellationToken cancellationToken)
        {
            var result = await _uploadService.GetStatusAsync(
                userId,
                id,
                cancellationToken);

            if (result is null)
                return NotFound();

            return Ok(result);
        }

        [HttpDelete("{id:guid}")]
        public async Task<IActionResult> Delete(
            Guid id,
            CancellationToken cancellationToken)
        {
            var deleted = await _uploadService.DeleteAsync(
                userId,
                id,
                cancellationToken);

            if (!deleted)
                return NotFound();

            return NoContent();

        }


        [HttpPut("{id:guid}/chunks/{chunkNumber:int}")]
        [Consumes("application/octet-stream")]
        public async Task<ActionResult<UploadChunkResponse>> UploadChunk(
            Guid id,
            int chunkNumber,
            CancellationToken cancellationToken)
        {
            UploadRequestLimits.Set(HttpContext, _uploadConfiguration.MaxChunkSize);

            if (!Request.ContentLength.HasValue)
            {
                return BadRequest(new
                {
                    error = "Content-Length header required"
                });
            }

            return await HandleUploadOperationAsync(() =>
                _uploadService.UploadChunkAsync(
                    userId,
                    id,
                    chunkNumber,
                    Request.Body,
                    Request.ContentLength.Value,
                    cancellationToken));

        }

        [HttpPost("{id:guid}/multipart/")]
        [Consumes("multipart/form-data")]
        [RequestSizeLimit(MultipartFormsLimit.MultipartFormMaxRequestBodySize)]
        public async Task<ActionResult<CompleteUploadResponse>> UploadMultipart(
            Guid id,
            CancellationToken cancellationToken)
        {
            if (!Request.HasFormContentType)
                return BadRequest(new
                {
                    error = "Request must use multipart/form-data."
                });


            var form = await Request.ReadFormAsync(
                new FormOptions
                {
                    MultipartBodyLengthLimit = MultipartFormsLimit.MultipartFormMaxRequestBodySize,
                    BufferBodyLengthLimit = MultipartFormsLimit.MultipartFormMaxRequestBodySize
                },
                cancellationToken
                );
            var file = form.Files["file"];
            if (file is null)
                return BadRequest(new
                {
                    error = "'file' field is required"
                });

            await using var stream = file.OpenReadStream();

            return await HandleUploadOperationAsync(() =>
                _uploadService.UploadMultipartAsync(
                    userId,
                    id,
                    stream,
                    file.Length,
                    cancellationToken));
        }

        [HttpPost("{id:guid}/stream")]
        [Consumes("application/octet-stream")]
        public async Task<ActionResult<CompleteUploadResponse>> UploadStream(
            Guid id,
            CancellationToken cancellationToken)
        {
            UploadRequestLimits.Set(HttpContext, _uploadConfiguration.MaxFileSize);

            if (!Request.ContentLength.HasValue)
            {
                return BadRequest(new
                {
                    error = "Content-Length header required"
                });
            }

            return await HandleUploadOperationAsync(() =>
                _uploadService.UploadStreamAsync(
                    userId,
                    id,
                    Request.Body,
                    Request.ContentLength.Value,
                    cancellationToken));


        }

        [HttpPost("{id:guid}/complete")]
        public async Task<ActionResult<CompleteUploadResponse>> Complete(
            Guid id,
            CancellationToken cancellationToken)
        {
            return await HandleUploadOperationAsync(() =>
                _uploadService.CompleteAsync(
                    userId,
                    id,
                    cancellationToken));

        }

        private async Task<ActionResult<T>> HandleUploadOperationAsync<T>(
            Func<Task<T>> operation)
        {
            try
            {
                var result = await operation();
                return Ok(result);
            }
            catch (KeyNotFoundException)
            {
                return NotFound();
            }
            catch (ArgumentException ex)
            {
                return BadRequest(new
                {
                    error = ex.Message,
                });
            }
            catch (InvalidOperationException ex)
            {
                return Conflict(new
                {
                    error = ex.Message,
                });
            }

        }
    }
}
