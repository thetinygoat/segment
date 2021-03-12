package filter

type EnglishFilter struct {
	stopwords    map[string]bool
	contractions map[string]bool
}

func NewEnglishFilter() *EnglishFilter {
	englishFilter := &EnglishFilter{}
	stopwords := make(map[string]bool)
	stopwords["hadn't"] = true
	stopwords["how"] = true
	stopwords["your"] = true
	stopwords["these"] = true
	stopwords["which"] = true
	stopwords["few"] = true
	stopwords["other"] = true
	stopwords["aren't"] = true
	stopwords["did"] = true
	stopwords["then"] = true
	stopwords["wasn't"] = true
	stopwords["if"] = true
	stopwords["wouldn't"] = true
	stopwords["won"] = true
	stopwords["you"] = true
	stopwords["on"] = true
	stopwords["again"] = true
	stopwords["was"] = true
	stopwords["we"] = true
	stopwords["such"] = true
	stopwords["that'll"] = true
	stopwords["re"] = true
	stopwords["couldn"] = true
	stopwords["hasn"] = true
	stopwords["can"] = true
	stopwords["won't"] = true
	stopwords["this"] = true
	stopwords["herself"] = true
	stopwords["him"] = true
	stopwords["very"] = true
	stopwords["so"] = true
	stopwords["between"] = true
	stopwords["here"] = true
	stopwords["are"] = true
	stopwords["doing"] = true
	stopwords["as"] = true
	stopwords["you'll"] = true
	stopwords["am"] = true
	stopwords["into"] = true
	stopwords["wouldn"] = true
	stopwords["ll"] = true
	stopwords["her"] = true
	stopwords["didn't"] = true
	stopwords["haven"] = true
	stopwords["shan"] = true
	stopwords["a"] = true
	stopwords["you've"] = true
	stopwords["weren"] = true
	stopwords["there"] = true
	stopwords["do"] = true
	stopwords["under"] = true
	stopwords["each"] = true
	stopwords["himself"] = true
	stopwords["some"] = true
	stopwords["isn"] = true
	stopwords["been"] = true
	stopwords["up"] = true
	stopwords["doesn't"] = true
	stopwords["is"] = true
	stopwords["ve"] = true
	stopwords["whom"] = true
	stopwords["i"] = true
	stopwords["has"] = true
	stopwords["hasn't"] = true
	stopwords["before"] = true
	stopwords["his"] = true
	stopwords["or"] = true
	stopwords["my"] = true
	stopwords["now"] = true
	stopwords["only"] = true
	stopwords["against"] = true
	stopwords["through"] = true
	stopwords["she's"] = true
	stopwords["while"] = true
	stopwords["nor"] = true
	stopwords["over"] = true
	stopwords["its"] = true
	stopwords["d"] = true
	stopwords["it"] = true
	stopwords["yours"] = true
	stopwords["it's"] = true
	stopwords["those"] = true
	stopwords["theirs"] = true
	stopwords["hadn"] = true
	stopwords["an"] = true
	stopwords["will"] = true
	stopwords["most"] = true
	stopwords["don"] = true
	stopwords["shan't"] = true
	stopwords["myself"] = true
	stopwords["and"] = true
	stopwords["by"] = true
	stopwords["same"] = true
	stopwords["in"] = true
	stopwords["does"] = true
	stopwords["when"] = true
	stopwords["should've"] = true
	stopwords["from"] = true
	stopwords["themselves"] = true
	stopwords["our"] = true
	stopwords["s"] = true
	stopwords["ain"] = true
	stopwords["having"] = true
	stopwords["the"] = true
	stopwords["any"] = true
	stopwords["until"] = true
	stopwords["ma"] = true
	stopwords["out"] = true
	stopwords["t"] = true
	stopwords["needn't"] = true
	stopwords["they"] = true
	stopwords["me"] = true
	stopwords["being"] = true
	stopwords["because"] = true
	stopwords["wasn"] = true
	stopwords["about"] = true
	stopwords["with"] = true
	stopwords["their"] = true
	stopwords["but"] = true
	stopwords["be"] = true
	stopwords["for"] = true
	stopwords["don't"] = true
	stopwords["ours"] = true
	stopwords["he"] = true
	stopwords["after"] = true
	stopwords["to"] = true
	stopwords["y"] = true
	stopwords["didn"] = true
	stopwords["isn't"] = true
	stopwords["that"] = true
	stopwords["mightn"] = true
	stopwords["who"] = true
	stopwords["m"] = true
	stopwords["aren"] = true
	stopwords["had"] = true
	stopwords["you're"] = true
	stopwords["off"] = true
	stopwords["at"] = true
	stopwords["doesn"] = true
	stopwords["shouldn"] = true
	stopwords["not"] = true
	stopwords["where"] = true
	stopwords["further"] = true
	stopwords["above"] = true
	stopwords["no"] = true
	stopwords["shouldn't"] = true
	stopwords["ourselves"] = true
	stopwords["own"] = true
	stopwords["mightn't"] = true
	stopwords["she"] = true
	stopwords["yourselves"] = true
	stopwords["below"] = true
	stopwords["you'd"] = true
	stopwords["itself"] = true
	stopwords["couldn't"] = true
	stopwords["all"] = true
	stopwords["hers"] = true
	stopwords["both"] = true
	stopwords["should"] = true
	stopwords["than"] = true
	stopwords["down"] = true
	stopwords["of"] = true
	stopwords["once"] = true
	stopwords["mustn't"] = true
	stopwords["too"] = true
	stopwords["have"] = true
	stopwords["yourself"] = true
	stopwords["o"] = true
	stopwords["haven't"] = true
	stopwords["what"] = true
	stopwords["them"] = true
	stopwords["mustn"] = true
	stopwords["needn"] = true
	stopwords["just"] = true
	stopwords["weren't"] = true
	stopwords["more"] = true
	stopwords["why"] = true
	stopwords["during"] = true
	stopwords["were"] = true
	englishFilter.stopwords = stopwords

	return englishFilter
}

func (e EnglishFilter) Punctuation(tokens []string) []string {
	var filtered []string

	for _, token := range tokens {
		switch token {
		case ".", "?", "!", ",", ":", ";", "-", "[", "]", "{", "}", "(", ")", "\"", "'", "—":
			continue
		default:
			filtered = append(filtered, token)
		}
	}

	return filtered
}

func (e *EnglishFilter) Stopwords(tokens []string) []string {
	var filtered []string

	for _, token := range tokens {
		if _, ok := e.stopwords[token]; ok {
			continue
		}
		filtered = append(filtered, token)
	}
	return filtered
}
