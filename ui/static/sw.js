// The service worker for Web Push (ADR 0107). The gateway encrypts each push to this browser;
// the browser decrypts it and hands this worker the notice: { kind, category, title, body, link }.
// It only shows notifications and opens their links: no caching, no fetch handling.
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (event) => event.waitUntil(self.clients.claim()));

self.addEventListener('push', (event) => {
  const notice = readNotice(event.data);
  const title = notice.title || 'Something new on Social Production';
  event.waitUntil(
    self.registration.showNotification(title, {
      body: notice.body || '',
      icon: '/icon-192.v4.png',
      badge: '/icon-monochrome-512.v4.png',
      // One notification per link: a newer push about the same thing replaces the older one.
      tag: notice.link || notice.kind || 'sp',
      data: { link: notice.link || '/notifications' },
    }),
  );
});

function readNotice(data) {
  try {
    return data ? data.json() : {};
  } catch {
    return {};
  }
}

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const link = new URL(event.notification.data?.link || '/notifications', self.location.origin);
  if (link.origin !== self.location.origin) return;
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((windows) => {
      const open = windows.find((w) => new URL(w.url).origin === link.origin);
      // navigate() works only on a page this worker controls; otherwise open a new one.
      if (open?.navigate)
        return open
          .navigate(link.href)
          .then((w) => (w ?? open).focus())
          .catch(() => self.clients.openWindow(link.href));
      return self.clients.openWindow(link.href);
    }),
  );
});
