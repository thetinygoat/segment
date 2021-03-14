package analyzer

import (
	"fmt"
	"testing"
)

func TestEnglishAnalyzer(t *testing.T) {
	tests := []struct {
		input string
		want  []string
	}{
		{
			"On a scale from one to ten, what's your favorite flavor of random grammar?",
			[]string{
				"scale", "one", "ten", "favorite", "flavor", "random", "grammar"},
		},
		{
			"The random sentence generator generated a random sentence about a random sentence.",
			[]string{
				"random", "sentence", "generator", "generated", "random", "sentence", "random", "sentence"},
		},
		{
			"He strives to keep the best lawn in the neighborhood.",
			[]string{
				"strives", "keep", "best", "lawn", "neighborhood"},
		},
		{
			"I don’t respect anybody who can’t tell the difference between Pepsi and Coke.",
			[]string{
				"respect", "anybody", "ca", "tell", "difference", "pepsi", "coke"},
		},
		{
			"Facing his greatest fear, he ate his first marshmallow.",
			[]string{
				"facing", "greatest", "fear", "ate", "first", "marshmallow"},
		},
		{
			"The quick brown fox jumps over the lazy dog.",
			[]string{
				"quick", "brown", "fox", "jumps", "lazy", "dog"},
		},
		{
			"His mind was blown that there was nothing in space except space itself.",
			[]string{
				"mind", "blown", "nothing", "space", "except", "space"},
		},
		{
			"He realized there had been several deaths on this road, but his concern rose when he saw the exact number.",
			[]string{
				"realized", "several", "deaths", "road", "concern", "rose", "saw", "exact", "number"},
		},
		{
			"Flesh-colored yoga pants were far worse than even he feared.",
			[]string{
				"flesh-colored", "yoga", "pants", "far", "worse", "even", "feared"},
		},
	}

	a := NewEnglishAnalyzer()
	for n, test := range tests {
		testName := fmt.Sprintf("english analyzer test %d", n+1)
		t.Run(testName, func(t *testing.T) {
			cleaned, err := a.Analyze(test.input)
			if err != nil {
				t.Errorf("unexpected error %s", err.Error())
			}
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
