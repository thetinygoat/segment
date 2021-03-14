package transformer

import (
	"fmt"
	"testing"
)

func TestLowercase(t *testing.T) {
	tests := []struct {
		input []string
		want  []string
	}{
		{
			[]string{"Her", "son", ",", "John", "Jones", "Jr.", ",", "was", "born", "on", "Dec.", "6", ",", "2008", "."},
			[]string{"her", "son", ",", "john", "jones", "jr.", ",", "was", "born", "on", "dec.", "6", ",", "2008", "."},
		},
		{
			[]string{"When", "did", "Jane", "leave", "for", "the", "market", "?"},
			[]string{"when", "did", "jane", "leave", "for", "the", "market", "?"},
		},
		{
			[]string{"\"", "Holy", "cow", "!", "\"", "screamed", "Jane", "."},
			[]string{"\"", "holy", "cow", "!", "\"", "screamed", "jane", "."},
		},
		{
			[]string{"John", "was", "hurt", ";", "he", "knew", "she", "only", "said", "it", "to", "upset", "him", "."},
			[]string{"john", "was", "hurt", ";", "he", "knew", "she", "only", "said", "it", "to", "upset", "him", "."},
		},
		{
			[]string{"He", "was", "planning", "to", "study", "four", "subjects", ":", "politics", ",", "philosophy", ",", "sociology", ",", "and", "economics", "."},
			[]string{"he", "was", "planning", "to", "study", "four", "subjects", ":", "politics", ",", "philosophy", ",", "sociology", ",", "and", "economics", "."},
		},
		{
			[]string{"He", "[", "Mr.", "Jones", "]", "was", "the", "last", "person", "seen", "at", "the", "house", "."},
			[]string{"he", "[", "mr.", "jones", "]", "was", "the", "last", "person", "seen", "at", "the", "house", "."},
		},
		{
			[]string{"John", "and", "Jane", "(", "who", "were", "actually", "half", "brother", "and", "sister", ")", "both", "have", "red", "hair", "."},
			[]string{"john", "and", "jane", "(", "who", "were", "actually", "half", "brother", "and", "sister", ")", "both", "have", "red", "hair", "."},
		},
		{
			[]string{"\"", "Do", "n't", "go", "outside", ",", "\"", "she", "said", "."},
			[]string{"\"", "do", "n't", "go", "outside", ",", "\"", "she", "said", "."},
		},
		{
			[]string{"He", "gave", "him", "her", "answer", "—", "No", "!"},
			[]string{"he", "gave", "him", "her", "answer", "—", "no", "!"},
		},
	}
	tf := NewEnglishTransformer()
	for n, test := range tests {
		testName := fmt.Sprintf("lowercase transformer test %d", n+1)

		t.Run(testName, func(t *testing.T) {
			cleaned := tf.Lowercase(test.input)
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
