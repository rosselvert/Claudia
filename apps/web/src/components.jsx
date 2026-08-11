import { useState } from "react";
import { Heart, Minus, Plus, ShoppingBag, UserRound, X } from "lucide-react";
import { Link, useNavigate } from "react-router-dom";
import { api, money } from "./api";
import { useStore } from "./StoreContext";

export function Brand() {
  return <Link to="/" className="text-lg font-semibold tracking-[.18em]">CLAUDIA<span className="text-rust">.</span></Link>;
}

export function StoreHeader() {
  const { user, cart, setAuthOpen, setCartOpen } = useStore();
  return <>
    <div className="bg-ink px-4 py-2 text-center text-[9px] tracking-[.1em] text-paper md:text-[11px]">Complimentary delivery across Indonesia for orders over Rp1.000.000</div>
    <header className="relative z-20 flex h-18 items-center justify-between border-b border-ink/15 px-5 md:h-20 md:px-[4vw]">
      <Brand />
      <nav className="absolute left-1/2 hidden -translate-x-1/2 gap-10 text-xs md:flex"><a href="/#shop">Shop</a><a href="/#story">Our approach</a></nav>
      <div className="flex items-center gap-2 md:gap-5">
        {user && <Link to="/account" className="icon-button" aria-label="Account"><UserRound size={18} /></Link>}
        {!user && <button onClick={() => setAuthOpen(true)} className="px-2 py-2 text-xs">Sign in</button>}
        <button onClick={() => setCartOpen(true)} className="flex items-center gap-2 px-1 py-2 text-xs"><ShoppingBag size={17} /><span className="hidden sm:inline">Bag</span><span className="grid size-6 place-items-center rounded-full bg-rust text-[10px] text-white">{cart.item_count}</span></button>
      </div>
    </header>
  </>;
}

export function Footer() {
  return <footer className="grid gap-10 bg-ink px-6 py-14 text-paper md:grid-cols-2 md:px-[4vw] md:py-18"><Brand /><p className="font-serif text-xl italic text-white/60 md:justify-self-end">Considered goods for everyday living.</p><div className="flex justify-between border-t border-white/15 pt-7 text-[9px] uppercase tracking-[.14em] text-white/45 md:col-span-2"><span>Jakarta, Indonesia</span><span>© 2026 Claudia</span></div></footer>;
}

export function AuthModal() {
  const { authOpen, setAuthOpen, authenticate } = useStore();
  const [mode, setMode] = useState("login");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  if (!authOpen) return null;
  async function submit(event) {
    event.preventDefault(); setBusy(true); setError("");
    const values = Object.fromEntries(new FormData(event.currentTarget));
    if (mode === "login") delete values.full_name;
    try { await authenticate(mode, values); } catch (failure) { setError(failure.message); } finally { setBusy(false); }
  }
  return <div className="fixed inset-0 z-50 grid place-items-center bg-ink/50 p-4 backdrop-blur-sm" onMouseDown={(event) => event.target === event.currentTarget && setAuthOpen(false)}>
    <section className="relative w-full max-w-md bg-paper p-7 shadow-2xl md:p-11">
      <button className="icon-button absolute right-3 top-3" onClick={() => setAuthOpen(false)}><X size={20} /></button>
      <p className="eyebrow mb-3">{mode === "login" ? "Welcome back" : "Join Claudia"}</p>
      <h2 className="serif-title text-4xl">{mode === "login" ? "Sign in to Claudia" : "Create your account"}</h2>
      <p className="my-4 font-serif text-lg text-ink/60">Access your bag, wishlist, and order history.</p>
      <form className="grid gap-4" onSubmit={submit}>
        {mode === "register" && <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Full name<input className="field" name="full_name" required minLength={2} /></label>}
        <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Email address<input className="field" name="email" type="email" required /></label>
        <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Password<input className="field" name="password" type="password" required minLength={8} /></label>
        <p className="min-h-4 text-xs text-rust">{error}</p>
        <button className="button-dark" disabled={busy}>{busy ? "Please wait..." : mode === "login" ? "Sign in" : "Create account"}</button>
      </form>
      <button className="mt-5 w-full text-xs text-ink/55 underline" onClick={() => { setMode(mode === "login" ? "register" : "login"); setError(""); }}>{mode === "login" ? "New here? Create an account" : "Already have an account? Sign in"}</button>
    </section>
  </div>;
}

export function CartDrawer() {
  const { user, cart, setCart, addresses, cartOpen, setCartOpen, setAuthOpen, updateQuantity, removeCartItem, notify } = useStore();
  const [checkout, setCheckout] = useState(false);
  const [selectedAddress, setSelectedAddress] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const navigate = useNavigate();
  if (!cartOpen) return null;
  const address = addresses.find((item) => item.id === selectedAddress);
  const shipping = cart.subtotal_cents >= 1_000_000 ? 0 : 30_000;
  const checkoutTotal = cart.subtotal_cents + shipping;
  function startCheckout() {
    setSelectedAddress(addresses.find((item) => item.is_default)?.id || "");
    setCheckout(true);
  }
  async function submitCheckout(event) {
    event.preventDefault(); setBusy(true); setError("");
    try {
      await api("/checkout", { method: "POST", body: JSON.stringify(Object.fromEntries(new FormData(event.currentTarget))) });
      setCart({ items: [], item_count: 0, subtotal_cents: 0 }); setCartOpen(false); setCheckout(false); notify("Your order is confirmed"); navigate("/account");
    } catch (failure) { setError(failure.message); } finally { setBusy(false); }
  }
  return <div className="fixed inset-0 z-50 bg-ink/40 backdrop-blur-[2px]" onMouseDown={(event) => event.target === event.currentTarget && setCartOpen(false)}>
    <aside className="absolute right-0 top-0 flex h-full w-full max-w-[480px] flex-col bg-paper shadow-2xl">
      <header className="flex items-start justify-between border-b border-ink/15 p-6"><div><p className="eyebrow mb-2">{checkout ? "Secure checkout" : "Your selection"}</p><h2 className="font-serif text-4xl">{checkout ? "Delivery details" : "Shopping bag"}</h2></div><button className="icon-button" onClick={() => checkout ? setCheckout(false) : setCartOpen(false)}><X size={21} /></button></header>
      {checkout ? <form key={selectedAddress || "manual"} className="grid flex-1 content-start gap-4 overflow-auto p-6" onSubmit={submitCheckout}>
        <p className="font-serif text-lg text-ink/60">Payment is simulated for this preview. Your order will be confirmed immediately.</p>
        {addresses.length > 0 && <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Saved address<select className="field" value={selectedAddress} onChange={(event) => setSelectedAddress(event.target.value)}><option value="">Enter a new address</option>{addresses.map((item) => <option key={item.id} value={item.id}>{item.label}{item.is_default ? " · Default" : ""}</option>)}</select></label>}
        <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Recipient name<input className="field" name="recipient_name" defaultValue={address?.recipient_name || user?.full_name} required minLength={2} /></label>
        <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Phone number<input className="field" name="phone" type="tel" defaultValue={address?.phone || ""} required minLength={7} /></label>
        <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Shipping address<textarea className="field min-h-28 resize-y" name="shipping_address" defaultValue={address?.address || ""} required minLength={10} /></label>
        <label className="grid gap-1.5 text-[9px] uppercase tracking-widest">Payment method<select className="field" name="payment_method" defaultValue="bank_transfer"><option value="bank_transfer">Bank transfer</option><option value="credit_card">Credit card · Demo instant payment</option><option value="cash_on_delivery">Cash on delivery</option></select></label>
        <div className="mt-3 grid gap-2 border-t border-ink/15 pt-5 text-xs"><div className="flex justify-between"><span>Subtotal</span><span>{money(cart.subtotal_cents)}</span></div><div className="flex justify-between"><span>Delivery</span><span>{shipping ? money(shipping) : "Complimentary"}</span></div><div className="mt-2 flex justify-between border-t border-ink/15 pt-4"><strong>Order total</strong><strong className="font-serif text-2xl">{money(checkoutTotal)}</strong></div></div>
        <p className="min-h-4 text-xs text-rust">{error}</p><button className="button-dark" disabled={busy}>{busy ? "Confirming..." : "Confirm order"}</button>
      </form> : <>
        <div className="flex-1 overflow-auto px-6">
          {!cart.items.length && <div className="grid h-full place-content-center text-center"><ShoppingBag className="mx-auto mb-4 text-ink/25" /><p className="font-serif text-3xl">{user ? "Nothing here yet." : "Your bag is waiting."}</p><span className="mt-2 text-xs text-ink/50">{user ? "Start with one good thing." : "Sign in to keep your selection."}</span></div>}
          {cart.items.map((item) => <article key={item.product_id} className="grid grid-cols-[80px_1fr_auto] gap-4 border-b border-ink/15 py-5">
            <img className="h-25 w-20 object-cover" src={item.image_url} alt={item.name} /><div><h3 className="font-serif text-xl">{item.name}</h3><p className="mt-1 text-xs text-ink/50">{money(item.price_cents)}</p><div className="mt-3 inline-flex items-center border border-ink/15"><button className="grid size-7 place-items-center" onClick={() => updateQuantity(item.product_id, item.quantity - 1)}><Minus size={12} /></button><span className="w-7 text-center text-xs">{item.quantity}</span><button className="grid size-7 place-items-center" disabled={item.quantity >= item.stock} onClick={() => updateQuantity(item.product_id, item.quantity + 1)}><Plus size={12} /></button></div></div><button className="self-start" onClick={() => removeCartItem(item.product_id)}><X size={17} /></button>
          </article>)}
        </div>
        <footer className="border-t border-ink/15 p-6"><div className="mb-1 flex justify-between"><span className="text-xs">Subtotal</span><strong className="font-serif text-2xl">{money(cart.subtotal_cents)}</strong></div><p className="mb-5 text-[10px] text-ink/45">Delivery calculated at checkout</p>{user ? <button className="button-dark w-full" disabled={!cart.items.length} onClick={startCheckout}>Continue to checkout</button> : <button className="button-dark w-full" onClick={() => { setCartOpen(false); setAuthOpen(true); }}>Sign in to continue</button>}</footer>
      </>}
    </aside>
  </div>;
}

export function ProductCard({ product, onOpen }) {
  const { wishlist, toggleWishlist, addToCart } = useStore();
  const saved = wishlist.some((item) => item.id === product.id);
  return <article className="group animate-enter">
    <div className="relative aspect-[4/5] overflow-hidden bg-cream">
      <button className="block h-full w-full" onClick={() => onOpen(product)}><img className="h-full w-full object-cover saturate-75 transition duration-700 group-hover:scale-[1.025] group-hover:saturate-100" src={product.image_url} alt={product.name} loading="lazy" /></button>
      {product.featured && <span className="absolute left-3 top-3 bg-paper px-2 py-1.5 text-[8px] uppercase tracking-[.15em]">Claudia pick</span>}
      <button className="absolute right-3 top-3 grid size-9 place-items-center rounded-full bg-paper/90" onClick={() => toggleWishlist(product)} aria-label="Toggle wishlist"><Heart size={16} fill={saved ? "currentColor" : "none"} className={saved ? "text-rust" : ""} /></button>
      <button className="absolute inset-x-3 bottom-3 translate-y-16 bg-ink/95 py-3 text-[9px] uppercase tracking-[.14em] text-white transition group-hover:translate-y-0 focus:translate-y-0 max-md:translate-y-0" disabled={!product.stock} onClick={() => addToCart(product.id)}>{product.stock ? "Add to bag" : "Sold out"}</button>
    </div>
    <div className="grid grid-cols-[1fr_auto] gap-1 pt-4"><h3 className="font-serif text-xl">{product.name}</h3><strong className="text-xs font-medium">{money(product.price_cents)}</strong><p className="text-[11px] text-ink/50">{product.category}</p></div>
  </article>;
}

export function ProductModal({ product, onClose }) {
  const { wishlist, toggleWishlist, addToCart } = useStore();
  if (!product) return null;
  const saved = wishlist.some((item) => item.id === product.id);
  return <div className="fixed inset-0 z-40 grid place-items-center bg-ink/50 p-4 backdrop-blur-sm" onMouseDown={(event) => event.target === event.currentTarget && onClose()}><section className="relative grid max-h-[92vh] w-full max-w-5xl overflow-auto bg-paper shadow-2xl md:grid-cols-2"><button className="icon-button absolute right-3 top-3 z-10 bg-paper/90" onClick={onClose}><X size={20} /></button><img className="h-[42vh] w-full object-cover saturate-75 md:h-full md:min-h-[590px]" src={product.image_url} alt={product.name} /><div className="self-center p-7 md:p-14"><p className="eyebrow mb-3">{product.category}</p><h2 className="serif-title text-5xl">{product.name}</h2><strong className="my-6 block font-serif text-2xl">{money(product.price_cents)}</strong><p className="font-serif text-lg leading-relaxed text-ink/60">{product.description}</p><p className="my-6 text-[10px] uppercase tracking-widest text-ink/60">{product.stock ? `${product.stock} pieces available` : "Currently unavailable"}</p><div className="flex gap-2"><button className="button-dark flex-1" disabled={!product.stock} onClick={() => addToCart(product.id)}>{product.stock ? "Add to bag" : "Sold out"}</button><button className="grid size-12 place-items-center border border-ink/20" onClick={() => toggleWishlist(product)}><Heart size={18} fill={saved ? "currentColor" : "none"} className={saved ? "text-rust" : ""} /></button></div></div></section></div>;
}

export function GlobalUi() {
  const { toast } = useStore();
  return <><AuthModal /><CartDrawer /><div className={`fixed bottom-6 left-1/2 z-[70] -translate-x-1/2 bg-ink px-5 py-3 text-xs text-white transition ${toast ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0"}`}>{toast}</div></>;
}
