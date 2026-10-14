
using System.Collections.Concurrent;

namespace Scrap_web.Services.Uploads
{
    public readonly record struct UploadLockKey(
        Guid UploadId, 
        int? ChunkNumber = null);
    public sealed class UploadLockManager : IUploadLockManager
    {
        private readonly ConcurrentDictionary<UploadLockKey, SemaphoreSlim> _locks = new();
        public async Task<IDisposable> AcquireAsync(
            Guid uploadId,
            CancellationToken cancellationToken = default)
        {
            var key = new UploadLockKey(uploadId);

            var semaphore = _locks.GetOrAdd(
                key,
                _ => new SemaphoreSlim(1, 1));

            await semaphore.WaitAsync(cancellationToken);

            return new Releaser(semaphore, key, _locks);
        }

        public async Task<IDisposable> AcquireAsync(
            Guid uploadId,
            int chunkNumber,
            CancellationToken cancellationToken = default) 
        {
            var key = new UploadLockKey(uploadId, chunkNumber);

            var semaphore = _locks.GetOrAdd(
                key,
                _ => new SemaphoreSlim(1, 1));

            await semaphore.WaitAsync(cancellationToken);

            return new Releaser(semaphore, key, _locks);
        }

        private sealed class Releaser : IDisposable
        {
            private readonly SemaphoreSlim _semaphore;
            private readonly UploadLockKey _uploadLockKey;
            private readonly ConcurrentDictionary<UploadLockKey, SemaphoreSlim> _locks;
            private bool _disposed;

            public Releaser(
                SemaphoreSlim semaphore,
                UploadLockKey uploadLockKey,
                ConcurrentDictionary<UploadLockKey, SemaphoreSlim> locks)
            {
                _semaphore = semaphore;
                _uploadLockKey = uploadLockKey;
                _locks = locks;                
            }

            public void Dispose()
            {
                if(_disposed) return;

                _semaphore.Release();

                _locks.TryRemove(
                    new KeyValuePair<UploadLockKey, SemaphoreSlim>(
                        _uploadLockKey,
                        _semaphore));

                _disposed = true;
            }
        }
    }
}
