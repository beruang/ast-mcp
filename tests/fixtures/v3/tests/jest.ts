// Jest/Vitest test fixtures
describe('UserService', () => {
  beforeEach(() => { setup(); });
  afterEach(() => { teardown(); });

  it('returns user by id', () => {
    const user = getUser('1');
    expect(user.id).toBe('1');
  });

  test('creates a new user', () => {
    const user = createUser({ name: 'Test' });
    expect(user.name).toBe('Test');
  });

  describe('edge cases', () => {
    it('handles missing user', () => {
      expect(() => getUser('999')).toThrow();
    });
  });
});
