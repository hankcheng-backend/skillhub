import "@testing-library/jest-dom";

// Mock @tauri-apps/api/core invoke
// All test files can override this per-test
const mockInvokeHandlers: Record<string, (...args: unknown[]) => unknown> = {};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    const handler = mockInvokeHandlers[cmd];
    if (handler) {
      return Promise.resolve(handler(args));
    }
    return Promise.reject(new Error(`Unhandled invoke: ${cmd}`));
  }),
}));

// Mock @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

// Mock @tauri-apps/plugin-opener
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

// Export for test files to register handlers
export function mockIPC(cmd: string, handler: (...args: unknown[]) => unknown) {
  mockInvokeHandlers[cmd] = handler;
}

export function clearMockIPC() {
  Object.keys(mockInvokeHandlers).forEach((k) => delete mockInvokeHandlers[k]);
}

// Auto-clear between tests
afterEach(() => {
  clearMockIPC();
});
