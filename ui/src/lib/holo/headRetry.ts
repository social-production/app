/**
 * Two writes by one person can land together (a second device, or a background write beside
 * a button press). Holochain refuses the later one before running it ("the source chain head
 * has moved since the bundle began"), so nothing was written and it is safe to send again.
 */
export async function withHeadRetry<T>(call: () => Promise<T>): Promise<T> {
  for (let attempt = 0; ; attempt++) {
    try {
      return await call();
    } catch (error) {
      // The client throws plain objects as well as Errors.
      const text = error instanceof Error ? error.message : JSON.stringify(error ?? '');
      if (attempt >= 3 || !/head has moved/i.test(text)) throw error;
      await new Promise((done) => setTimeout(done, 200 * (attempt + 1)));
    }
  }
}
