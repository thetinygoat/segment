package text

import "testing"

func TestFilterPuncChar(t *testing.T) {
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
			res := FilterPuncChar(tc.input)
			if res != tc.want {
				t.Errorf("got %s, want %s", res, tc.want)
			}
		})
	}
}

func benchmarFilterPuncChar(input string, b *testing.B) {
	for i := 0; i < b.N; i++ {
		FilterPuncChar(input)
	}
}

func BenchmarkFilterPuncChar1(b *testing.B) {
	benchmarFilterPuncChar("Jane and Jack went to the market.", b)
}

func BenchmarkFilterPuncChar2(b *testing.B) {
	benchmarFilterPuncChar("John was hurt; he knew she only said it to upset him.", b)
}

func BenchmarkFilterPuncChar3(b *testing.B) {
	benchmarFilterPuncChar("John and Jane (who were actually half brother and sister) both have red hair.", b)
}
