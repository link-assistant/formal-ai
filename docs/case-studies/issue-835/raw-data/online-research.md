# Online research for issue 835

Research was checked on 2026-08-02. Sources are primary standards bodies,
government-linked reporting infrastructure, official provider documentation,
and the upstream metadata library.

## Jurisdiction and copyright scope

- WIPO's [Copyright FAQ](https://www.wipo.int/en/web/copyright/faq-copyright)
  explains that copyright protection is territorial and recommends researching
  the laws of the countries concerned. This rules out a context-free global
  clearance result.
- WIPO's [copyright overview](https://www.wipo.int/en/web/copyright/) states
  that WIPO does not offer a searchable global copyright database and that
  protection is automatic in the majority of countries. Missing reverse-search
  or registry evidence therefore cannot establish permission.
- The [Berne Convention overview](https://www.wipo.int/en/web/treaties/ip/berne/index)
  is the treaty-level source for minimum protection and national treatment; it
  does not replace national policy packs.

## Child-safety hash boundary

- Microsoft's [PhotoDNA overview](https://www.microsoft.com/en-us/photodna)
  describes a non-reversible image signature compared with signatures of known
  illegal images, and distinguishes that mechanism from facial or object
  recognition.
- Microsoft's [PhotoDNA Cloud Service](https://www.microsoft.com/en-us/photodna/CloudService?oneroute=true)
  limits use to qualified organizations subject to approval and describes a
  provider flow that matches, flags, and reports. Formal AI therefore models a
  generic authorized-provider receipt rather than bundling a database or
  claiming provider authorization.
- NCMEC's [CyberTipline data page](https://cf.missingkids.org/gethelpnow/cybertipline/cybertiplinedata)
  describes hash matching and provider reporting at ecosystem scale. The local
  report retains only a provider case reference/report channel and fails
  closed.

## Metadata

- CIPA publishes the [Exif standards](https://www.cipa.jp/e/std/std-sec.html),
  including Exif 3.1 and the relationship with embedded XMP metadata.
- The upstream [`kamadak-exif` repository](https://github.com/kamadak/exif-rs)
  documents supported image containers and its read-only Rust API. The
  implementation pins version 0.6.1 and extracts only selected bounded fields.

## Engineering conclusion

The evidence supports a category-level, jurisdiction-relative pipeline with
versioned sources and explicit uncertainty. It does not support a single
`legal: true` result. External object/symbol/reverse-search systems belong
behind adapters; restricted hash databases remain solely with authorized
providers.
