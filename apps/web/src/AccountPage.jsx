import { useEffect, useState } from "react";
import { Heart, MapPin, PackageOpen, Plus, X } from "lucide-react";
import { Link } from "react-router-dom";
import { api, money, shortDate } from "./api";
import { ProductCard, ProductModal, StoreHeader } from "./components";
import { useStore } from "./StoreContext";

export default function AccountPage() {
  const { user, setUser, sessionLoading, wishlist, addresses, saveAddress, deleteAddress, logout, notify } = useStore();
  const [tab, setTab] = useState("orders");
  const [orders, setOrders] = useState([]);
  const [detail, setDetail] = useState(null);
  const [selectedProduct, setSelectedProduct] = useState(null);
  const [profileError, setProfileError] = useState("");
  const [addressEditor, setAddressEditor] = useState(null);

  useEffect(() => {
    if (user) api("/orders").then(setOrders).catch(() => setOrders([]));
  }, [user]);

  if (sessionLoading) return <div className="grid min-h-screen place-items-center font-serif text-3xl">Opening your account...</div>;
  if (!user) return <><StoreHeader /><main className="grid min-h-[70vh] place-content-center px-5 text-center"><p className="eyebrow mb-4">Your Claudia account</p><h1 className="serif-title text-6xl">Come back in.</h1><p className="mx-auto my-5 max-w-md font-serif text-xl text-ink/60">Sign in from the storefront to see your details, wishlist, and orders.</p><Link className="button-dark mx-auto" to="/">Return to storefront</Link></main></>;

  async function openOrder(orderId) {
    setDetail({ loading: true });
    try { setDetail(await api(`/orders/${orderId}`)); } catch (error) { setDetail({ error: error.message }); }
  }

  async function saveProfile(event) {
    event.preventDefault(); setProfileError("");
    const button = event.currentTarget.querySelector("button"); button.disabled = true;
    try {
      const nextUser = await api("/me", { method: "PATCH", body: JSON.stringify(Object.fromEntries(new FormData(event.currentTarget))) });
      setUser(nextUser);
    } catch (error) { setProfileError(error.message); } finally { button.disabled = false; }
  }

  async function changePassword(event) {
    event.preventDefault();
    const form = event.currentTarget;
    const values = Object.fromEntries(new FormData(form));
    const errorElement = form.querySelector("[data-error]");
    errorElement.textContent = "";
    if (values.new_password !== values.confirm_password) {
      errorElement.textContent = "New password confirmation does not match";
      return;
    }
    delete values.confirm_password;
    const button = form.querySelector("button");
    button.disabled = true;
    try {
      const session = await api("/me/password", { method: "POST", body: JSON.stringify(values) });
      localStorage.setItem("claudia_token", session.token);
      form.reset();
      notify("Password updated and other sessions signed out");
    } catch (error) { errorElement.textContent = error.message; } finally { button.disabled = false; }
  }

  return <>
    <StoreHeader />
    <main className="mx-auto max-w-[1440px] px-5 py-14 md:px-[4vw] md:py-20">
      <header className="flex items-end justify-between border-b border-ink/15 pb-12"><div><p className="eyebrow mb-4">Your account</p><h1 className="serif-title text-6xl md:text-8xl">Hello, <em className="text-sage">{user.full_name.split(" ")[0]}.</em></h1></div><div className="hidden text-right text-xs leading-relaxed md:block"><span className="text-ink/50">Signed in as</span><strong className="block font-serif text-xl">{user.email}</strong><button className="mt-2 underline" onClick={logout}>Sign out</button></div></header>
      <div className="grid gap-12 pt-12 md:grid-cols-[210px_1fr] md:gap-[7vw]">
        <nav className="flex h-fit gap-4 overflow-auto border-b border-ink/15 md:flex-col md:gap-1 md:border-0">{["orders", "wishlist", "addresses", "profile", "security"].map((item) => <button key={item} className={`shrink-0 border-b px-0 py-3 text-left text-sm capitalize transition md:w-full ${tab === item ? "border-ink text-ink" : "border-transparent text-ink/45"}`} onClick={() => setTab(item)}>{item}</button>)}{user.role === "admin" && <Link className="shrink-0 py-3 text-sm text-rust" to="/admin">Admin dashboard</Link>}</nav>
        <section>
          {tab === "orders" && <><SectionTitle eyebrow="Purchase history" title="Your orders" meta={`${orders.length} total`} />{orders.length ? <div>{orders.map((order) => <button key={order.id} onClick={() => openOrder(order.id)} className="grid w-full grid-cols-[1fr_auto] items-center gap-4 border-t border-ink/15 py-5 text-left md:grid-cols-[1.2fr_.8fr_auto]"><div><h3 className="font-serif text-xl">Order #{order.id.slice(0, 8)}</h3><p className="mt-1 text-[11px] text-ink/50">{shortDate(order.created_at)} · {order.payment_method.replaceAll("_", " ")} · {order.payment_status}</p></div><span className="font-serif text-lg md:order-none">{money(order.total_cents)}</span><span className="col-start-2 row-start-2 justify-self-end rounded-full bg-sage/15 px-3 py-1.5 text-[8px] uppercase tracking-widest text-sage md:col-auto md:row-auto">{order.status}</span></button>)}</div> : <Empty icon={PackageOpen} title="Your first order awaits." text="Explore our collection of considered everyday pieces." />}</>}
          {tab === "wishlist" && <><SectionTitle eyebrow="Saved for later" title="Your wishlist" meta={`${wishlist.length} pieces`} />{wishlist.length ? <div className="grid grid-cols-2 gap-3 md:grid-cols-3 md:gap-5">{wishlist.map((product) => <ProductCard key={product.id} product={product} onOpen={setSelectedProduct} />)}</div> : <Empty icon={Heart} title="Keep an eye on something." text="Tap the heart on any product to keep it here." />}</>}
          {tab === "addresses" && <><SectionTitle eyebrow="Delivery details" title="Address book" meta={`${addresses.length} saved`} /><button className="button-dark mb-7" onClick={() => setAddressEditor({})}><Plus size={14} /> Add address</button>{addresses.length ? <div className="grid gap-3 sm:grid-cols-2">{addresses.map((address) => <article key={address.id} className="relative border border-ink/15 p-5"><div className="mb-4 flex items-center gap-2"><MapPin size={17} className="text-rust" /><h3 className="font-serif text-2xl">{address.label}</h3>{address.is_default && <span className="ml-auto rounded-full bg-sage/15 px-2 py-1 text-[8px] uppercase tracking-widest text-sage">Default</span>}</div><strong className="text-sm">{address.recipient_name}</strong><p className="mt-1 text-xs text-ink/50">{address.phone}</p><p className="mt-3 text-sm leading-relaxed">{address.address}</p><div className="mt-5 flex gap-4 text-[10px] underline"><button onClick={() => setAddressEditor(address)}>Edit</button><button onClick={() => deleteAddress(address.id)}>Remove</button></div></article>)}</div> : <Empty icon={MapPin} title="No saved addresses." text="Save an address to move through checkout faster." />}</>}
          {tab === "profile" && <><SectionTitle eyebrow="Personal details" title="Your profile" /><div className="max-w-xl border-t border-ink/15 text-sm"><ProfileRow label="Email" value={user.email} /><ProfileRow label="Account type" value={user.role} /></div><form onSubmit={saveProfile} className="mt-8 grid max-w-xl gap-3 sm:grid-cols-[1fr_auto] sm:items-end"><label className="grid gap-2 text-[9px] uppercase tracking-widest">Full name<input className="field" name="full_name" defaultValue={user.full_name} required minLength={2} maxLength={100} /></label><button className="button-dark">Save profile</button><p className="text-xs text-rust sm:col-span-2">{profileError}</p></form></>}
          {tab === "security" && <><SectionTitle eyebrow="Account protection" title="Security" /><p className="mb-7 max-w-xl font-serif text-lg leading-relaxed text-ink/55">Changing your password signs out every other device and rotates your active session.</p><form onSubmit={changePassword} className="grid max-w-xl gap-4"><PasswordField label="Current password" name="current_password" autoComplete="current-password" /><PasswordField label="New password" name="new_password" autoComplete="new-password" /><PasswordField label="Confirm new password" name="confirm_password" autoComplete="new-password" /><p data-error className="min-h-4 text-xs text-rust" /><button className="button-dark sm:justify-self-start">Update password</button></form></>}
        </section>
      </div>
    </main>
    {detail && <OrderModal detail={detail} onClose={() => setDetail(null)} />}
    <ProductModal product={selectedProduct} onClose={() => setSelectedProduct(null)} />
    {addressEditor && <AddressModal address={addressEditor.id ? addressEditor : null} onClose={() => setAddressEditor(null)} onSave={saveAddress} />}
  </>;
}

function SectionTitle({ eyebrow, title, meta }) {
  return <header className="mb-7 flex items-end justify-between"><div><p className="eyebrow mb-3">{eyebrow}</p><h2 className="font-serif text-4xl">{title}</h2></div>{meta && <span className="text-[10px] text-ink/45">{meta}</span>}</header>;
}

function ProfileRow({ label, value }) {
  return <div className="grid grid-cols-[130px_1fr] border-b border-ink/15 py-5"><span className="text-[9px] uppercase tracking-widest text-ink/45">{label}</span><strong>{value}</strong></div>;
}

function PasswordField({ label, name, autoComplete }) {
  return <label className="grid gap-2 text-[9px] uppercase tracking-widest">{label}<input className="field" name={name} type="password" autoComplete={autoComplete} required minLength={8} maxLength={72} /></label>;
}

function Empty({ icon: Icon, title, text }) {
  return <div className="border-t border-ink/15 py-20 text-center"><Icon className="mx-auto mb-4 text-ink/20" /><h3 className="font-serif text-3xl">{title}</h3><p className="mt-2 text-xs text-ink/50">{text}</p><Link to="/#shop" className="mt-5 inline-block border-b border-ink pb-1 text-[9px] uppercase tracking-widest">Browse collection</Link></div>;
}

function OrderModal({ detail, onClose }) {
  return <div className="fixed inset-0 z-40 grid place-items-center bg-ink/50 p-4 backdrop-blur-sm" onMouseDown={(event) => event.target === event.currentTarget && onClose()}><section className="relative w-full max-w-xl bg-paper p-7 shadow-2xl md:p-10"><button className="icon-button absolute right-3 top-3" onClick={onClose}><X size={20} /></button><p className="eyebrow mb-3">Order detail</p>{detail.loading ? <h2 className="font-serif text-3xl">Loading...</h2> : detail.error ? <p className="text-rust">{detail.error}</p> : <><h2 className="font-serif text-4xl">Order #{detail.order.id.slice(0, 8)}</h2><p className="mt-2 text-xs capitalize text-ink/50">{detail.order.payment_method.replaceAll("_", " ")} · Payment {detail.order.payment_status}</p><div className="mt-6">{detail.items.map((item) => <div key={item.product_id} className="flex justify-between border-t border-ink/15 py-4"><div><strong className="font-serif text-lg">{item.product_name}</strong><p className="text-[11px] text-ink/45">{item.quantity} × {money(item.unit_price_cents)}</p></div><strong>{money(item.subtotal_cents)}</strong></div>)}</div><div className="mt-4 grid gap-2 border-t border-ink pt-5 text-xs"><div className="flex justify-between"><span>Subtotal</span><span>{money(detail.order.subtotal_cents)}</span></div><div className="flex justify-between"><span>Delivery</span><span>{detail.order.shipping_cents ? money(detail.order.shipping_cents) : "Complimentary"}</span></div><div className="mt-2 flex justify-between font-serif text-2xl"><span>Total</span><strong>{money(detail.order.total_cents)}</strong></div></div></>}</section></div>;
}

function AddressModal({ address, onClose, onSave }) {
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function submit(event) {
    event.preventDefault(); setBusy(true); setError("");
    const form = event.currentTarget;
    const values = Object.fromEntries(new FormData(form));
    values.is_default = form.is_default.checked;
    try { await onSave(values, address?.id); onClose(); } catch (failure) { setError(failure.message); } finally { setBusy(false); }
  }
  return <div className="fixed inset-0 z-50 grid place-items-center bg-ink/50 p-4 backdrop-blur-sm" onMouseDown={(event) => event.target === event.currentTarget && onClose()}><section className="relative w-full max-w-lg bg-paper p-7 shadow-2xl md:p-10"><button className="icon-button absolute right-3 top-3" onClick={onClose}><X size={20} /></button><p className="eyebrow mb-3">Address book</p><h2 className="font-serif text-4xl">{address ? "Edit address" : "Add address"}</h2><form className="mt-6 grid gap-4" onSubmit={submit}><label className="grid gap-2 text-[9px] uppercase tracking-widest">Label<input className="field" name="label" defaultValue={address?.label || "Home"} required maxLength={40} /></label><label className="grid gap-2 text-[9px] uppercase tracking-widest">Recipient name<input className="field" name="recipient_name" defaultValue={address?.recipient_name || ""} required minLength={2} maxLength={100} /></label><label className="grid gap-2 text-[9px] uppercase tracking-widest">Phone number<input className="field" name="phone" type="tel" defaultValue={address?.phone || ""} required minLength={7} maxLength={30} /></label><label className="grid gap-2 text-[9px] uppercase tracking-widest">Full address<textarea className="field min-h-24 resize-y" name="address" defaultValue={address?.address || ""} required minLength={10} maxLength={500} /></label><label className="flex items-center gap-2 text-xs"><input name="is_default" type="checkbox" defaultChecked={address?.is_default} /> Use as default address</label><p className="min-h-4 text-xs text-rust">{error}</p><button className="button-dark" disabled={busy}>{busy ? "Saving..." : "Save address"}</button></form></section></div>;
}
