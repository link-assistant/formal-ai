using System.Collections.Generic;
using System.Numerics;
using System.Runtime.CompilerServices;
using Platform.Converters;
using Platform.Data.Doublets.Sequences.Indexes;

#pragma warning disable CS1591 // Missing XML comment for publicly visible type or member

namespace Platform.Data.Doublets.Sequences.Unicode
{
    /// <summary>
    /// <para>
    /// Represents the string to unicode sequence converter.
    /// </para>
    /// <para></para>
    /// </summary>
    /// <seealso cref="LinksOperatorBase{TLinkAddress}"/>
    /// <seealso cref="IConverter{string, TLinkAddress}"/>
    public class StringToUnicodeSequenceConverter<TLinkAddress> : LinksOperatorBase<TLinkAddress>, IConverter<string, TLinkAddress> where TLinkAddress : struct, IUnsignedNumber<TLinkAddress>, IComparisonOperators<TLinkAddress, TLinkAddress, bool>
    {
        private readonly IConverter<string, IList<TLinkAddress>?> _stringToUnicodeSymbolListConverter;
        private readonly IConverter<IList<TLinkAddress>, TLinkAddress> _unicodeSymbolListToSequenceConverter;

        /// <summary>
        /// <para>
        /// Initializes a new <see cref="StringToUnicodeSequenceConverter"/> instance.
        /// </para>
        /// <para></para>
        /// </summary>
        /// <param name="links">
        /// <para>A links.</para>
        /// <para></para>
        /// </param>
        /// <param name="stringToUnicodeSymbolListConverter">
        /// <para>A string to unicode symbol list converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="unicodeSymbolListToSequenceConverter">
        /// <para>A unicode symbol list to sequence converter.</para>
        /// <para></para>
        /// </param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public StringToUnicodeSequenceConverter(ILinks<TLinkAddress> links, IConverter<string, IList<TLinkAddress>?> stringToUnicodeSymbolListConverter, IConverter<IList<TLinkAddress>, TLinkAddress> unicodeSymbolListToSequenceConverter) : base(links)
        {
            _stringToUnicodeSymbolListConverter = stringToUnicodeSymbolListConverter;
            _unicodeSymbolListToSequenceConverter = unicodeSymbolListToSequenceConverter;
        }

        /// <summary>
        /// <para>
        /// Initializes a new <see cref="StringToUnicodeSequenceConverter"/> instance.
        /// </para>
        /// <para></para>
        /// </summary>
        /// <param name="links">
        /// <para>A links.</para>
        /// <para></para>
        /// </param>
        /// <param name="stringToUnicodeSymbolListConverter">
        /// <para>A string to unicode symbol list converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="index">
        /// <para>A index.</para>
        /// <para></para>
        /// </param>
        /// <param name="listToSequenceLinkConverter">
        /// <para>A list to sequence link converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="unicodeSequenceMarker">
        /// <para>A unicode sequence marker.</para>
        /// <para></para>
        /// </param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public StringToUnicodeSequenceConverter(ILinks<TLinkAddress> links, IConverter<string, IList<TLinkAddress>?> stringToUnicodeSymbolListConverter, ISequenceIndex<TLinkAddress> index, IConverter<IList<TLinkAddress>, TLinkAddress> listToSequenceLinkConverter, TLinkAddress unicodeSequenceMarker)
            : this(links, stringToUnicodeSymbolListConverter, new UnicodeSymbolsListToUnicodeSequenceConverter<TLinkAddress>(links, index, listToSequenceLinkConverter, unicodeSequenceMarker)) { }

        /// <summary>
        /// <para>
        /// Initializes a new <see cref="StringToUnicodeSequenceConverter"/> instance.
        /// </para>
        /// <para></para>
        /// </summary>
        /// <param name="links">
        /// <para>A links.</para>
        /// <para></para>
        /// </param>
        /// <param name="charToUnicodeSymbolConverter">
        /// <para>A char to unicode symbol converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="index">
        /// <para>A index.</para>
        /// <para></para>
        /// </param>
        /// <param name="listToSequenceLinkConverter">
        /// <para>A list to sequence link converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="unicodeSequenceMarker">
        /// <para>A unicode sequence marker.</para>
        /// <para></para>
        /// </param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public StringToUnicodeSequenceConverter(ILinks<TLinkAddress> links, IConverter<char, TLinkAddress> charToUnicodeSymbolConverter, ISequenceIndex<TLinkAddress> index, IConverter<IList<TLinkAddress>, TLinkAddress> listToSequenceLinkConverter, TLinkAddress unicodeSequenceMarker)
            : this(links, new StringToUnicodeSymbolsListConverter<TLinkAddress>(charToUnicodeSymbolConverter), index, listToSequenceLinkConverter, unicodeSequenceMarker) { }

        /// <summary>
        /// <para>
        /// Initializes a new <see cref="StringToUnicodeSequenceConverter"/> instance.
        /// </para>
        /// <para></para>
        /// </summary>
        /// <param name="links">
        /// <para>A links.</para>
        /// <para></para>
        /// </param>
        /// <param name="charToUnicodeSymbolConverter">
        /// <para>A char to unicode symbol converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="listToSequenceLinkConverter">
        /// <para>A list to sequence link converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="unicodeSequenceMarker">
        /// <para>A unicode sequence marker.</para>
        /// <para></para>
        /// </param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public StringToUnicodeSequenceConverter(ILinks<TLinkAddress> links, IConverter<char, TLinkAddress> charToUnicodeSymbolConverter, IConverter<IList<TLinkAddress>, TLinkAddress> listToSequenceLinkConverter, TLinkAddress unicodeSequenceMarker)
            : this(links, charToUnicodeSymbolConverter, new Unindex<TLinkAddress>(), listToSequenceLinkConverter, unicodeSequenceMarker) { }

        /// <summary>
        /// <para>
        /// Initializes a new <see cref="StringToUnicodeSequenceConverter"/> instance.
        /// </para>
        /// <para></para>
        /// </summary>
        /// <param name="links">
        /// <para>A links.</para>
        /// <para></para>
        /// </param>
        /// <param name="stringToUnicodeSymbolListConverter">
        /// <para>A string to unicode symbol list converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="listToSequenceLinkConverter">
        /// <para>A list to sequence link converter.</para>
        /// <para></para>
        /// </param>
        /// <param name="unicodeSequenceMarker">
        /// <para>A unicode sequence marker.</para>
        /// <para></para>
        /// </param>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public StringToUnicodeSequenceConverter(ILinks<TLinkAddress> links, IConverter<string, IList<TLinkAddress>?> stringToUnicodeSymbolListConverter, IConverter<IList<TLinkAddress>, TLinkAddress> listToSequenceLinkConverter, TLinkAddress unicodeSequenceMarker)
            : this(links, stringToUnicodeSymbolListConverter, new Unindex<TLinkAddress>(), listToSequenceLinkConverter, unicodeSequenceMarker) { }

        /// <summary>
        /// <para>
        /// Converts the source.
        /// </para>
        /// <para></para>
        /// </summary>
        /// <param name="source">
        /// <para>The source.</para>
        /// <para></para>
        /// </param>
        /// <returns>
        /// <para>The link</para>
        /// <para></para>
        /// </returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public TLinkAddress Convert(string source)
        {
            var elements = _stringToUnicodeSymbolListConverter.Convert(source);
            return _unicodeSymbolListToSequenceConverter.Convert(elements);
        }
    }
}
