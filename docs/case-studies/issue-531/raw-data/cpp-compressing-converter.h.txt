namespace Platform::Data::Doublets::Sequences::Converters
{
    template <typename ...> class CompressingConverter;
    template <typename TLink> class CompressingConverter<TLink> : public LinksListToSequenceConverterBase<TLink>
    {
        private: static readonly LinksConstants<TLink> _constants = Default<LinksConstants<TLink>>.Instance;

        private: inline static const TLink _zero = 0;
        private: inline static const TLink _one = _zero + 1;

        private: readonly IConverter<IList<TLink>, TLink> _baseConverter;
        private: LinkFrequenciesCache<TLink> _doubletFrequenciesCache;
        private: TLink _minFrequencyToCompress = 0;
        private: bool _doInitialFrequenciesIncrement = 0;
        private: Doublet<TLink> _maxDoublet;
        private: LinkFrequency<TLink> _maxDoubletData;

        struct HalfDoublet
        {
            public: TLink Element = 0;
            public: LinkFrequency<TLink> DoubletData;

            public: HalfDoublet(TLink element, LinkFrequency<TLink> doubletData)
            {
                Element = element;
                DoubletData = doubletData;
            }

            public: override std::string ToString() { return std::string("").append(Platform::Converters::To<std::string>(Element)).append(": (").append(Platform::Converters::To<std::string>(DoubletData)).append(1, ')'); }
        }

        public: CompressingConverter(ILinks<TLink> &links, IConverter<IList<TLink>, TLink> baseConverter, LinkFrequenciesCache<TLink> doubletFrequenciesCache)
            : this(links, baseConverter, doubletFrequenciesCache, _one, true) { }

        public: CompressingConverter(ILinks<TLink> &links, IConverter<IList<TLink>, TLink> baseConverter, LinkFrequenciesCache<TLink> doubletFrequenciesCache, bool doInitialFrequenciesIncrement)
            : this(links, baseConverter, doubletFrequenciesCache, _one, doInitialFrequenciesIncrement) { }

        public: CompressingConverter(ILinks<TLink> &links, IConverter<IList<TLink>, TLink> baseConverter, LinkFrequenciesCache<TLink> doubletFrequenciesCache, TLink minFrequencyToCompress, bool doInitialFrequenciesIncrement)
            : LinksListToSequenceConverterBase(links)
        {
            _baseConverter = baseConverter;
            _doubletFrequenciesCache = doubletFrequenciesCache;
            if (minFrequencyToCompress < _one)
            {
                minFrequencyToCompress = _one;
            }
            _minFrequencyToCompress = minFrequencyToCompress;
            _doInitialFrequenciesIncrement = doInitialFrequenciesIncrement;
            this->ResetMaxDoublet();
        }

        public: TLink Convert(IList<TLink> &source) override { return _baseConverter.Convert(this->Compress(source)); }

        private: IList<TLink> Compress(IList<TLink> &sequence)
        {
            if (sequence.IsNullOrEmpty())
            {
                return {};
            }
            if (sequence.Count() == 1)
            {
                return sequence;
            }
            if (sequence.Count() == 2)
            {
                return new[] { _links.GetOrCreate(sequence[0], sequence[1]) };
            }
            auto copy = HalfDoublet[sequence.Count()];
            Doublet<TLink> doublet = 0;
            for (auto i = 1; i < sequence.Count(); i++)
            {
                doublet = Doublet<TLink>(sequence[i - 1], sequence[i]);
                LinkFrequency<TLink> data;
                if (_doInitialFrequenciesIncrement)
                {
                    data = _doubletFrequenciesCache.IncrementFrequency(doublet);
                }
                else
                {
                    data = _doubletFrequenciesCache.GetFrequency(doublet);
                    if (data == nullptr)
                    {
                        throw NotSupportedException("If you ask not to increment frequencies, it is expected that all frequencies for the sequence are prepared.");
                    }
                }
                copy[i - 1].Element = sequence[i - 1];
                copy[i - 1].DoubletData = data;
                UpdateMaxDoublet(doublet, data);
            }
            copy[sequence.Count() - 1].Element = sequence[sequence.Count() - 1];
            copy[sequence.Count() - 1].DoubletData = LinkFrequency<TLink>();
            if (_maxDoubletData.Frequency > 0)
            {
                auto newLength = ReplaceDoublets(copy);
                sequence = TLink[newLength];
                for (std::int32_t i = 0; i < newLength; i++)
                {
                    sequence[i] = copy[i].Element;
                }
            }
            return sequence;
        }

        private: std::int32_t ReplaceDoublets(HalfDoublet copy[])
        {
            auto oldLength = copy.Length;
            auto newLength = copy.Length;
            while (_maxDoubletData.Frequency > 0)
            {
                auto maxDoubletSource = _maxDoublet.Source;
                auto maxDoubletTarget = _maxDoublet.Target;
                if (_maxDoubletData.Link == _constants.Null)
                {
                    _maxDoubletData.Link = _links.GetOrCreate(maxDoubletSource, maxDoubletTarget);
                }
                auto maxDoubletReplacementLink = _maxDoubletData.Link;
                oldLength--;
                auto oldLengthMinusTwo = oldLength - 1;
                std::int32_t w = 0, r = 0;
                for (; r < oldLength; r++)
                {
                    if (copy[r].Element == maxDoubletSource && copy[r + 1].Element == maxDoubletTarget)
                    {
                        if (r > 0)
                        {
                            auto previous = copy[w - 1].Element;
                            copy[w - 1].DoubletData.DecrementFrequency();
                            copy[w - 1].DoubletData = _doubletFrequenciesCache.IncrementFrequency(previous, maxDoubletReplacementLink);
                        }
                        if (r < oldLengthMinusTwo)
                        {
                            auto next = copy[r + 2].Element;
                            copy[r + 1].DoubletData.DecrementFrequency();
                            copy[w].DoubletData = _doubletFrequenciesCache.IncrementFrequency(maxDoubletReplacementLink, next);
                        }
                        copy[w++].Element = maxDoubletReplacementLink;
                        r++;
                        newLength--;
                    }
                    else
                    {
                        copy[w++] = copy[r];
                    }
                }
                if (w < newLength)
                {
                    copy[w] = copy[r];
                }
                oldLength = newLength;
                this->ResetMaxDoublet();
                this->UpdateMaxDoublet(copy, newLength);
            }
            return newLength;
        }

        private: void ResetMaxDoublet()
        {
            _maxDoublet = Doublet<TLink>();
            _maxDoubletData = LinkFrequency<TLink>();
        }

        private: void UpdateMaxDoublet(HalfDoublet copy[], std::int32_t length)
        {
            Doublet<TLink> doublet = 0;
            for (auto i = 1; i < length; i++)
            {
                doublet = Doublet<TLink>(copy[i - 1].Element, copy[i].Element);
                this->UpdateMaxDoublet(doublet, copy[i - 1].DoubletData);
            }
        }

        private: void UpdateMaxDoublet(ref Doublet<TLink> doublet, LinkFrequency<TLink> data)
        {
            auto frequency = data.Frequency;
            auto maxFrequency = _maxDoubletData.Frequency;
            if (frequency > _minFrequencyToCompress &&
               (maxFrequency < frequency || (maxFrequency == frequency && _comparer.Compare((doublet.Source) + (doublet.Target), (_maxDoublet.Source) + (_maxDoublet.Target)) > 0))) /* gives better stability and better compression on sequent data and even on rundom numbers data (but gives collisions anyway) */
            {
                _maxDoublet = doublet;
                _maxDoubletData = data;
            }
        }
    };
}