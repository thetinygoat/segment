package analyzer

import (
	"github.com/thetinygoat/segment/pkg/filter"
	"github.com/thetinygoat/segment/pkg/tokenizer"
	"github.com/thetinygoat/segment/pkg/transformer"
)

type EnglishAnalyzer struct {
}

func NewEnglishAnalyzer() *EnglishAnalyzer {
	return &EnglishAnalyzer{}
}

func (a *EnglishAnalyzer) Analyze(input string) ([]string, error) {
	to := tokenizer.NewEnglishTokenizer()
	tf := transformer.NewEnglishTransformer()
	ft := filter.NewEnglishFilter()

	rawTokens, err := to.Tokenize(input)
	if err != nil {
		return nil, err
	}

	lowercaseTransformed := tf.Lowercase(rawTokens)

	punctuationFiltered := ft.Punctuation(lowercaseTransformed)
	stopwordFiletred := ft.Stopwords(punctuationFiltered)

	return stopwordFiletred, nil
}
