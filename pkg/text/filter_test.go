package text

import "testing"

func TestPuncFilter(t *testing.T) {
	var data = []struct {
		input string
		want  string
	}{
		{"Jane and Jack went to the market.", "Jane and Jack went to the market"},
		{"Her son, John Jones Jr., was born on Dec. 6, 2008", "Her son John Jones Jr was born on Dec 6 2008"},
		{"When did Jane leave for the market?", "When did Jane leave for the market"},
		{"\"Holy cow!\" screamed Jane.", "Holy cow screamed Jane"},
		{"John was hurt; he knew she only said it to upset him.", "John was hurt he knew she only said it to upset him"},
		{"", ""},
		{"I didn't have time to get changed: I was already late.", "I didnt have time to get changed I was already late"},
		{"She gave him her answer — No!", "She gave him her answer  No"},
		{"He [Mr. Jones] was the last person seen at the house.", "He Mr Jones was the last person seen at the house"},
		{"John and Jane (who were actually half brother and sister) both have red hair.", "John and Jane who were actually half brother and sister both have red hair"},
		{"Sara's dog bit the neighbor.", "Saras dog bit the neighbor"},
	}

	for _, tc := range data {
		name := tc.input
		t.Run(name, func(t *testing.T) {
			res := PuncFilter(tc.input)
			if res != tc.want {
				t.Errorf("got %s, want %s", res, tc.want)
			}
		})
	}
}

func benchmarPuncFilter(input string, b *testing.B) {
	for i := 0; i < b.N; i++ {
		PuncFilter(input)
	}
}

func BenchmarkPuncFilter1(b *testing.B) {
	benchmarPuncFilter("Jane and Jack went to the market.", b)
}

func BenchmarkPuncFilter2(b *testing.B) {
	benchmarPuncFilter("John was hurt; he knew she only said it to upset him.", b)
}

func BenchmarkPuncFilter3(b *testing.B) {
	benchmarPuncFilter("John and Jane (who were actually half brother and sister) both have red hair.", b)
}
