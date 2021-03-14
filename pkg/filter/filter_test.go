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
	f := NewEnglishFilter()
	for n, test := range tests {
		testName := fmt.Sprintf("punctuation filter test %d", n+1)
		t.Run(testName, func(t *testing.T) {
			cleaned := f.Punctuation(test.input)
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

func TestStopwords(t *testing.T) {
	tests := []struct {
		input []string
		want  []string
	}{
		{
			[]string{"her", "son", "John", "Jones", "Jr.", "was", "born", "on", "Dec.", "6", "2008"},
			[]string{"son",
				"John",
				"Jones",
				"Jr.",
				"born",
				"Dec.",
				"6",
				"2008"},
		},
		{
			[]string{"when", "did", "Jane", "leave", "for", "the", "market"},
			[]string{"Jane",
				"leave",
				"market"},
		},
		{
			[]string{"Holy", "cow", "screamed", "Jane"},
			[]string{"Holy",
				"cow",
				"screamed",
				"Jane"},
		},
		{
			[]string{"John", "was", "hurt", "he", "knew", "she", "only", "said", "it", "to", "upset", "him"},
			[]string{"John",
				"hurt",
				"knew",
				"said",
				"upset"},
		},
		{
			[]string{"he", "was", "planning", "to", "study", "four", "subjects", "politics", "philosophy", "sociology", "and", "economics"},
			[]string{"planning",
				"study",
				"four",
				"subjects",
				"politics",
				"philosophy",
				"sociology",
				"economics"},
		},
		{
			[]string{"he", "Mr.", "Jones", "was", "the", "last", "person", "seen", "at", "the", "house"},
			[]string{"Mr.",
				"Jones",
				"last",
				"person",
				"seen",
				"house"},
		},
		{
			[]string{"John", "and", "Jane", "who", "were", "actually", "half", "brother", "and", "sister", "both", "have", "red", "hair"},
			[]string{"John",
				"Jane",
				"actually",
				"half",
				"brother",
				"sister",
				"red",
				"hair"},
		},
		{
			[]string{"do", "n't", "go", "outside", "she", "said"},
			[]string{
				"go",
				"outside",
				"said"},
		},
		{
			[]string{"he", "gave", "him", "her", "answer", "No"},
			[]string{"gave",
				"answer",
				"No"},
		},
		{
			[]string{"he", "'s", "not", "coming", "with", "us"},
			[]string{"coming",
				"us",
			},
		},
		{
			[]string{"i’ve", "decided", "to", "go", "to", "the", "party", "after", "all"},
			[]string{"i’ve",
				"decided",
				"go",
				"party",
			},
		},
		{
			[]string{"it", "'s", "his", "birthday", "and", "he", "has", "other", "plans"},
			[]string{"birthday",
				"plans",
			},
		},
		{
			[]string{"they've", "thought", "about", "going", "to", "the", "movies"},
			[]string{"they've",
				"thought",
				"going",
				"movies",
			},
		},
		{
			[]string{"are", "n't", "you", "Caroline", "'s", "friend"},
			[]string{
				"Caroline",
				"friend",
			},
		},
		{
			[]string{"it", "looked", "as", "if", "she'd", "already", "made", "up", "her", "mind"},
			[]string{"looked",
				"she'd",
				"already",
				"made",
				"mind",
			},
		},
	}

	f := NewEnglishFilter()

	for n, test := range tests {
		testName := fmt.Sprintf("stopword filter test %d", n+1)
		t.Run(testName, func(t *testing.T) {
			cleaned := f.Stopwords(test.input)
			if len(test.want) != len(cleaned) {
				t.Errorf("expected length %d, got %d", len(test.want), len(cleaned))
			}

			for i := range cleaned {
				if test.want[i] != cleaned[i] {
					t.Errorf("expected %s, got %s", test.want[i], cleaned[i])
				}
			}
		})
	}
}
