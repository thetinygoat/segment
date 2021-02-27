package text

import (
	"testing"
)

func TestWhitespaceTokenizer(t *testing.T) {
	var data = []struct {
		input string
		want  []string
	}{
		{"Jane and Jack went to the market", []string{"Jane", "and", "Jack", "went", "to", "the", "market"}},
		{"Her son John Jones Jr was born on Dec 6 2008", []string{"Her", "son", "John", "Jones", "Jr", "was", "born", "on", "Dec", "6", "2008"}},
		{"When did Jane leave for the market", []string{"When", "did", "Jane", "leave", "for", "the", "market"}},
		{"Holy cow screamed Jane", []string{"Holy", "cow", "screamed", "Jane"}},
		{"John was hurt he knew she only said it to upset him", []string{"John", "was", "hurt", "he", "knew", "she", "only", "said", "it", "to", "upset", "him"}},
		{"", []string{}},
		{"I didnt have time to get changed I was already late", []string{"I", "didnt", "have", "time", "to", "get", "changed", "I", "was", "already", "late"}},
		{"She gave him her answer  No", []string{"She", "gave", "him", "her", "answer", "No"}},
		{"               ", []string{}},
	}

	for _, tc := range data {
		name := tc.input
		t.Run(name, func(t *testing.T) {
			res := WhitespaceTokenizer(tc.input)
			if len(res) != len(tc.want) {
				t.Errorf("want length %d, got %d", len(tc.want), len(res))
				return
			}
			for i := 0; i < len(tc.want); i++ {
				if res[i] != tc.want[i] {
					t.Errorf("want %s, got %s", tc.want[i], res[i])
					break
				}
			}
		})
	}
}

func benchmarkWhitespaceTokenizer(input string, b *testing.B) {
	for i := 0; i < b.N; i++ {
		WhitespaceTokenizer(input)
	}
}

func BenchmarkWhitespaceTokenizer1(b *testing.B) {
	benchmarkWhitespaceTokenizer("Jane and Jack went to the market", b)
}

func BenchmarkWhitespaceTokenizer2(b *testing.B) {
	benchmarkWhitespaceTokenizer("She gave him her answer  No", b)
}

func BenchmarkWhitespaceTokenizer3(b *testing.B) {
	benchmarkWhitespaceTokenizer("Her son John Jones Jr was born on Dec 6 2008", b)
}

func BenchmarkWhitespaceTokenizer4(b *testing.B) {
	benchmarkWhitespaceTokenizer("               ", b)
}
