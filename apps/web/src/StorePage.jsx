import { useEffect, useState } from "react";
import { Search } from "lucide-react";
import { api } from "./api";
import { Footer, ProductCard, ProductModal, StoreHeader } from "./components";

const categories = ["", "Home", "Apparel", "Accessories", "Electronics", "Stationery"];

export default function StorePage() {
  const [products, setProducts] = useState([]);
  const [category, setCategory] = useState("");
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    const timer = window.setTimeout(() => {
      const params = new URLSearchParams();
      if (category) params.set("category", category);
      if (search.trim()) params.set("search", search.trim());
      setLoading(true); setError("");
      api(`/products?${params}`).then(setProducts).catch((failure) => setError(failure.message)).finally(() => setLoading(false));
    }, search ? 300 : 0);
    return () => window.clearTimeout(timer);
  }, [category, search]);

  return <>
    <StoreHeader />
    <main>
      <section className="grid min-h-[680px] gap-8 px-5 pb-16 pt-10 md:grid-cols-[.88fr_1.12fr] md:gap-[5vw] md:px-[4vw] md:pb-20">
        <div className="animate-enter self-center py-8 md:pl-[4vw]"><p className="eyebrow mb-5">The August edit</p><h1 className="serif-title text-[clamp(58px,7vw,112px)]">Objects for a life<br /><em className="text-sage">well considered.</em></h1><p className="my-7 max-w-md font-serif text-xl leading-relaxed text-ink/65">A concise collection of useful, enduring pieces for home, work, and everywhere between.</p><a className="inline-flex gap-10 border-b border-ink py-3 text-[10px] font-semibold uppercase tracking-[.16em]" href="#shop">Explore the collection <span>→</span></a></div>
        <div className="animate-enter relative min-h-[440px] overflow-hidden"><img className="absolute inset-0 h-full w-full object-cover saturate-75" src="https://images.unsplash.com/photo-1618221195710-dd6b41faaea6?auto=format&fit=crop&w=1800&q=85" alt="A calm thoughtfully furnished interior" /><span className="absolute bottom-4 left-4 bg-paper/90 px-3 py-2 text-[8px] uppercase tracking-[.15em]">Home, made personal</span></div>
      </section>
      <section id="story" className="mx-5 flex flex-col gap-6 border-y border-ink/15 py-7 md:mx-[4vw] md:flex-row md:items-center md:justify-between"><p className="font-serif text-2xl italic">Less, but better.</p><div className="flex flex-wrap gap-5 text-[9px] uppercase tracking-[.12em] md:gap-9"><span><b className="mr-2 text-rust">01</b>Useful by design</span><span><b className="mr-2 text-rust">02</b>Made to stay</span><span><b className="mr-2 text-rust">03</b>Honest materials</span></div></section>
      <section id="shop" className="px-5 py-24 md:px-[4vw] md:py-32">
        <header className="mb-12 flex flex-col gap-8 md:flex-row md:items-end md:justify-between"><div><p className="eyebrow mb-4">Shop the collection</p><h2 className="serif-title text-5xl md:text-7xl">Considered essentials</h2></div><label className="flex w-full border-b border-ink py-2 md:w-80"><input className="w-full bg-transparent text-sm outline-none" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search the collection" /><Search size={17} /></label></header>
        <div className="mb-8 flex items-center justify-between gap-5"><div className="flex gap-2 overflow-auto pb-2">{categories.map((item) => <button key={item || "All"} onClick={() => setCategory(item)} className={`shrink-0 rounded-full border px-4 py-2 text-[10px] transition ${category === item ? "border-ink bg-ink text-white" : "border-ink/15 hover:border-ink"}`}>{item || "All"}</button>)}</div><span className="hidden text-[10px] text-ink/45 sm:block">{products.length} pieces</span></div>
        {error && <div className="py-20 text-center font-serif text-3xl">{error}</div>}
        <div className="grid min-h-96 grid-cols-2 gap-x-3 gap-y-10 md:grid-cols-3 md:gap-x-5 md:gap-y-14">{loading ? Array.from({ length: 6 }, (_, index) => <div key={index} className="animate-pulse"><div className="aspect-[4/5] bg-cream" /><div className="mt-4 h-4 w-2/3 bg-cream" /></div>) : products.map((product) => <ProductCard key={product.id} product={product} onOpen={setSelected} />)}</div>
        {!loading && !error && !products.length && <div className="py-24 text-center"><h3 className="font-serif text-4xl">Nothing found.</h3><p className="mt-2 text-sm text-ink/50">Try another search or category.</p></div>}
      </section>
      <section className="grid bg-[#d9d0c0] md:grid-cols-[1.15fr_.85fr]"><div className="m-5 min-h-[55vh] overflow-hidden md:m-12"><img className="h-full w-full object-cover saturate-75" src="https://images.unsplash.com/photo-1494438639946-1ebd1d20bf85?auto=format&fit=crop&w=1600&q=85" alt="Natural materials in a bright studio" /></div><div className="self-center px-6 py-16 md:p-16"><p className="eyebrow mb-4">The Claudia standard</p><h2 className="serif-title text-5xl md:text-7xl">Good things should earn their place.</h2><p className="my-7 font-serif text-xl leading-relaxed text-ink/65">We look for honest materials, thoughtful details, and utility that quietly improves your day. No endless aisles. Just pieces we believe in.</p></div></section>
    </main>
    <Footer />
    <ProductModal product={selected} onClose={() => setSelected(null)} />
  </>;
}
