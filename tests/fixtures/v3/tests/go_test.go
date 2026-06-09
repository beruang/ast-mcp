package fixtures

import "testing"

func TestGetUser(t *testing.T) {
	user := GetUser("1")
	if user.ID != "1" {
		t.Errorf("expected ID 1, got %s", user.ID)
	}
}

func BenchmarkUserLookup(b *testing.B) {
	for i := 0; i < b.N; i++ {
		GetUser("1")
	}
}

func FuzzUserParsing(f *testing.F) {
	f.Add("test")
	f.Fuzz(func(t *testing.T, input string) {
		ParseUser(input)
	})
}
