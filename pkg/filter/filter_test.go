package filter

import (
	"fmt"
	"testing"
)

func TestPunctuation(t *testing.T) {
	tests := []struct {
		input []string
		want  []string
	}{
		{
			[]string{"Her", "son", ",", "John", "Jones", "Jr.", ",", "was", "born", "on", "Dec.", "6", ",", "2008", "."},
			[]string{"Her", "son", "John", "Jones", "Jr.", "was", "born", "on", "Dec.", "6", "2008"},
		},
		{
			[]string{"When", "did", "Jane", "leave", "for", "the", "market", "?"},
			[]string{"When", "did", "Jane", "leave", "for", "the", "market"},
		},
		{
			[]string{"\"", "Holy", "cow", "!", "\"", "screamed", "Jane", "."},
			[]string{"Holy", "cow", "screamed", "Jane"},
		},
		{
			[]string{"John", "was", "hurt", ";", "he", "knew", "she", "only", "said", "it", "to", "upset", "him", "."},
			[]string{"John", "was", "hurt", "he", "knew", "she", "only", "said", "it", "to", "upset", "him"},
		},
		{
			[]string{"He", "was", "planning", "to", "study", "four", "subjects", ":", "politics", ",", "philosophy", ",", "sociology", ",", "and", "economics", "."},
			[]string{"He", "was", "planning", "to", "study", "four", "subjects", "politics", "philosophy", "sociology", "and", "economics"},
		},
		{
			[]string{"He", "[", "Mr.", "Jones", "]", "was", "the", "last", "person", "seen", "at", "the", "house", "."},
			[]string{"He", "Mr.", "Jones", "was", "the", "last", "person", "seen", "at", "the", "house"},
		},
		{
			[]string{"John", "and", "Jane", "(", "who", "were", "actually", "half", "brother", "and", "sister", ")", "both", "have", "red", "hair", "."},
			[]string{"John", "and", "Jane", "who", "were", "actually", "half", "brother", "and", "sister", "both", "have", "red", "hair"},
		},
		{
			[]string{"\"", "Do", "n't", "go", "outside", ",", "\"", "she", "said", "."},
			[]string{"Do", "n't", "go", "outside", "she", "said"},
		},
		{
			[]string{"he", "gave", "him", "her", "answer", "—", "No", "!"},
			[]string{"he", "gave", "him", "her", "answer", "No"},
		},
	}

	for n, test := range tests {
		testName := fmt.Sprintf("punctuation filter test %d", n+1)
		t.Run(testName, func(t *testing.T) {
			cleaned := Punctuation(test.input)
			if len(cleaned) != len(test.want) {
				t.Errorf("expected length %d, got %d", len(test.want), len(cleaned))
			}
			for i := range cleaned {
				if test.want[i] != cleaned[i] {
					t.Errorf("expected token %s, got %s", test.want[i], cleaned[i])
				}
			}
		})
	}

}
