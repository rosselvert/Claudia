import { useEffect, useState } from "react";
import { Boxes, ChartNoAxesCombined, CircleAlert, Plus, ShoppingCart, UsersRound, X } from "lucide-react";
import { Link } from "react-router-dom";
import { api, money, shortDate } from "./api";
import { Brand } from "./components";
import { useStore } from "./StoreContext";

const views = [{ id: "overview", icon: ChartNoAxesCombined }, { id: "products", icon: Boxes }, { id: "orders", icon: ShoppingCart }, { id: "customers", icon: UsersRound }];

export default function AdminPage() {
  const { user, sessionLoading, setAuthOpen } = useStore();
  const [view, setView] = useState("overview");
  const [data, setData] = useState({ metrics: null, products: [], orders: [], customers: [] });
  const [productModal, setProductModal] = useState(null);
  const [orderDetail, setOrderDetail] = useState(null);
  const [search, setSearch] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (user?.role !== "admin") return;
    Promise.all([api("/admin/metrics"), api("/admin/products"), api("/admin/orders"), api("/admin/customers")])
      .then(([metrics, products, orders, customers]) => setData({ metrics, products, orders, customers }))
      .catch((failure) => setError(failure.message));
  }, [user]);

  if (sessionLoading) return <div className="grid min-h-screen place-items-center font-serif text-3xl">Opening operations...</div>;
  if (!user || user.role !== "admin") return <main className="grid min-h-screen place-content-center bg-paper px-5 text-center"><p className="eyebrow mb-4">Restricted area</p><h1 className="serif-title text-6xl">Admin access required.</h1><p className="mx-auto my-5 max-w-md font-serif text-xl text-ink/55">Sign in with an administrator account to manage Claudia.</p><div className="mx-auto flex gap-2"><button className="button-dark" onClick={() => setAuthOpen(true)}>Sign in as admin</button><Link to="/" className="border border-ink px-5 py-4 text-[10px] font-semibold uppercase tracking-[.16em]">Storefront</Link></div></main>;

  async function setOrderStatus(orderId, status) {
    try {
      const order = await api(`/admin/orders/${orderId}/status`, { method: "PATCH", body: JSON.stringify({ status }) });
      setData((current) => ({ ...current, orders: current.orders.map((item) => item.id === order.id ? order : item) }));
    } catch (failure) { window.alert(failure.message); }
  }

  async function setPaymentStatus(orderId, status) {
    try {
      const order = await api(`/admin/orders/${orderId}/payment`, { method: "PATCH", body: JSON.stringify({ status }) });
      setData((current) => ({ ...current, orders: current.orders.map((item) => item.id === order.id ? order : item) }));
    } catch (failure) { window.alert(failure.message); }
  }

  async function setRole(customerId, role) {
    try {
      const customer = await api(`/admin/customers/${customerId}/role`, { method: "PATCH", body: JSON.stringify({ role }) });
      setData((current) => ({ ...current, customers: current.customers.map((item) => item.id === customer.id ? customer : item) }));
    } catch (failure) { window.alert(failure.message); }
  }

  async function archiveProduct(productId) {
    if (!window.confirm("Archive this product from the storefront?")) return;
    try { await api(`/admin/products/${productId}`, { method: "DELETE" }); setData((current) => ({ ...current, products: current.products.map((item) => item.id === productId ? { ...item, active: false } : item) })); } catch (failure) { window.alert(failure.message); }
  }

  async function openOrder(orderId) {
    setOrderDetail({ loading: true });
    try { setOrderDetail(await api(`/admin/orders/${orderId}`)); } catch (failure) { setOrderDetail({ error: failure.message }); }
  }

  const title = view[0].toUpperCase() + view.slice(1);
  return <div className="min-h-screen bg-[#ece9e1] md:grid md:grid-cols-[220px_1fr]">
    <aside className="flex items-center bg-ink px-4 py-4 text-paper md:sticky md:top-0 md:h-screen md:flex-col md:items-stretch md:px-6 md:py-8"><Brand /><nav className="ml-auto flex md:ml-0 md:mt-14 md:grid md:gap-1">{views.map(({ id, icon: Icon }, index) => <button key={id} title={id} onClick={() => { setView(id); setSearch(""); }} className={`flex items-center gap-3 rounded-sm p-3 text-left text-sm capitalize transition ${view === id ? "bg-white/10 text-white" : "text-white/45 hover:text-white"}`}><span className="hidden w-5 text-[9px] md:block">0{index + 1}</span><Icon size={17} className="md:hidden" /><span className="hidden md:block">{id}</span></button>)}</nav><div className="mt-auto hidden border-t border-white/15 pt-5 text-[10px] text-white/45 md:block"><strong className="block text-xs text-white">{user.full_name}</strong>{user.email}</div></aside>
    <main className="min-w-0 p-4 pb-16 md:p-10 lg:px-16">
      <header className="mb-9 flex items-center justify-between"><div><p className="eyebrow mb-2">Claudia operations</p><h1 className="serif-title text-5xl">{title}</h1><p className="mt-2 text-[11px] text-ink/45">{new Intl.DateTimeFormat("id-ID", { weekday: "long", day: "numeric", month: "long" }).format(new Date())}</p></div><Link to="/account" className="button-dark">My account</Link></header>
      {error && <p className="mb-5 bg-rust p-3 text-sm text-white">{error}</p>}
      {view === "overview" && <Overview metrics={data.metrics} orders={data.orders} onViewOrders={() => setView("orders")} onOpen={openOrder} />}
      {view === "products" && <Products products={data.products} search={search} setSearch={setSearch} onEdit={setProductModal} onArchive={archiveProduct} onNew={() => setProductModal({})} />}
      {view === "orders" && <Orders orders={data.orders} search={search} setSearch={setSearch} onStatus={setOrderStatus} onPayment={setPaymentStatus} onOpen={openOrder} />}
      {view === "customers" && <Customers customers={data.customers} currentUser={user.id} search={search} setSearch={setSearch} onRole={setRole} />}
    </main>
    {productModal && <ProductForm product={productModal.id ? productModal : null} onClose={() => setProductModal(null)} onSaved={(product) => { setData((current) => ({ ...current, products: current.products.some((item) => item.id === product.id) ? current.products.map((item) => item.id === product.id ? product : item) : [product, ...current.products] })); setProductModal(null); }} />}
    {orderDetail && <AdminOrderModal detail={orderDetail} onClose={() => setOrderDetail(null)} />}
  </div>;
}

function Overview({ metrics, orders, onViewOrders, onOpen }) {
  const cards = metrics ? [
    ["Gross revenue", money(metrics.revenue_cents), ChartNoAxesCombined], ["Orders", metrics.order_count, ShoppingCart], ["Customers", metrics.customer_count, UsersRound], ["Low stock", metrics.low_stock_count, CircleAlert],
  ] : [];
  return <><div className="mb-5 grid grid-cols-2 gap-2 lg:grid-cols-4">{cards.map(([label, value, Icon]) => <article key={label} className="panel flex min-h-32 flex-col justify-between"><div className="flex justify-between"><span className="text-[9px] uppercase tracking-widest text-ink/45">{label}</span><Icon size={16} className="text-ink/25" /></div><strong className="font-serif text-3xl">{value}</strong></article>)}</div><div className="panel"><PanelHeader title="Recent orders"><button className="text-[10px] underline" onClick={onViewOrders}>View all</button></PanelHeader><OrderTable orders={orders.slice(0, 6)} onOpen={onOpen} /></div></>;
}

function Products({ products, search, setSearch, onEdit, onArchive, onNew }) {
  const shown = products.filter((item) => `${item.name} ${item.category} ${item.slug}`.toLowerCase().includes(search.toLowerCase()));
  return <div className="panel"><PanelHeader title="Catalog"><div className="flex gap-2"><SearchInput value={search} onChange={setSearch} placeholder="Search products" /><button className="button-dark" onClick={onNew}><Plus size={14} /> Add product</button></div></PanelHeader><div className="overflow-auto"><table className="w-full min-w-175 text-left text-xs"><thead><tr className="border-b border-ink/15 text-[8px] uppercase tracking-widest text-ink/45"><th className="p-3">Product</th><th>Category</th><th>Price</th><th>Stock</th><th>Visibility</th><th /></tr></thead><tbody>{shown.map((product) => <tr key={product.id} className="border-b border-ink/10"><td className="p-3"><div className="flex items-center gap-3"><img className="h-12 w-10 object-cover" src={product.image_url} alt="" /><div><strong>{product.name}</strong><span className="block text-[10px] text-ink/40">{product.slug}</span></div></div></td><td>{product.category}</td><td>{money(product.price_cents)}</td><td className={product.stock <= 5 ? "text-rust" : ""}>{product.stock}</td><td>{product.active ? "Visible" : "Archived"}{product.featured && " · Featured"}</td><td className="whitespace-nowrap"><button className="mr-3 underline" onClick={() => onEdit(product)}>Edit</button><button className="underline" onClick={() => onArchive(product.id)}>Archive</button></td></tr>)}</tbody></table></div></div>;
}

function Orders({ orders, search, setSearch, onStatus, onPayment, onOpen }) {
  const shown = orders.filter((item) => `${item.id} ${item.customer_name} ${item.customer_email}`.toLowerCase().includes(search.toLowerCase()));
  return <div className="panel"><PanelHeader title="All orders"><SearchInput value={search} onChange={setSearch} placeholder="Search orders" /></PanelHeader><OrderTable orders={shown} onStatus={onStatus} onPayment={onPayment} onOpen={onOpen} /></div>;
}

function OrderTable({ orders, onStatus, onPayment, onOpen }) {
  return <div className="overflow-auto"><table className="w-full min-w-200 text-left text-xs"><thead><tr className="border-b border-ink/15 text-[8px] uppercase tracking-widest text-ink/45"><th className="p-3">Order</th><th>Customer</th><th>Fulfillment</th><th>Payment</th><th>Total</th><th>Date</th></tr></thead><tbody>{orders.map((order) => <tr key={order.id} className="border-b border-ink/10"><td className="p-3"><button className="font-semibold underline" onClick={() => onOpen?.(order.id)}>#{order.id.slice(0, 8)}</button></td><td>{order.customer_name}<span className="block text-[10px] text-ink/40">{order.customer_email}</span></td><td>{onStatus ? <select className="border border-ink/15 bg-transparent p-2 text-[10px]" value={order.status} onChange={(event) => onStatus(order.id, event.target.value)}>{["confirmed", "processing", "shipped", "delivered", "cancelled"].map((status) => <option key={status}>{status}</option>)}</select> : <span className="rounded-full bg-sage/15 px-2 py-1 text-[8px] uppercase">{order.status}</span>}</td><td>{onPayment ? <select className="border border-ink/15 bg-transparent p-2 text-[10px]" value={order.payment_status} onChange={(event) => onPayment(order.id, event.target.value)}>{["pending", "paid", "refunded"].map((status) => <option key={status}>{status}</option>)}</select> : <span className="capitalize">{order.payment_status}</span>}<span className="mt-1 block text-[9px] capitalize text-ink/40">{order.payment_method.replaceAll("_", " ")}</span></td><td>{money(order.total_cents)}</td><td>{shortDate(order.created_at)}</td></tr>)}</tbody></table></div>;
}

function Customers({ customers, currentUser, search, setSearch, onRole }) {
  const shown = customers.filter((item) => `${item.full_name} ${item.email}`.toLowerCase().includes(search.toLowerCase()));
  return <div className="panel"><PanelHeader title="Customers"><SearchInput value={search} onChange={setSearch} placeholder="Search customers" /></PanelHeader><div className="overflow-auto"><table className="w-full min-w-175 text-left text-xs"><thead><tr className="border-b border-ink/15 text-[8px] uppercase tracking-widest text-ink/45"><th className="p-3">Customer</th><th>Role</th><th>Orders</th><th>Total spent</th><th>Joined</th></tr></thead><tbody>{shown.map((customer) => <tr key={customer.id} className="border-b border-ink/10"><td className="p-3"><strong>{customer.full_name}</strong><span className="block text-[10px] text-ink/40">{customer.email}</span></td><td><select className="border border-ink/15 bg-transparent p-2 text-[10px]" disabled={customer.id === currentUser} value={customer.role} onChange={(event) => onRole(customer.id, event.target.value)}><option>user</option><option>admin</option></select></td><td>{customer.order_count}</td><td>{money(customer.total_spent_cents)}</td><td>{shortDate(customer.created_at)}</td></tr>)}</tbody></table></div></div>;
}

function ProductForm({ product, onClose, onSaved }) {
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function submit(event) {
    event.preventDefault(); setBusy(true); setError("");
    const form = event.currentTarget, values = Object.fromEntries(new FormData(form));
    values.price_cents = Number(values.price_cents); values.stock = Number(values.stock); values.featured = form.featured.checked; values.active = form.active.checked;
    try { onSaved(await api(product ? `/admin/products/${product.id}` : "/admin/products", { method: product ? "PUT" : "POST", body: JSON.stringify(values) })); } catch (failure) { setError(failure.message); } finally { setBusy(false); }
  }
  return <div className="fixed inset-0 z-50 grid place-items-center bg-ink/50 p-3 backdrop-blur-sm"><section className="relative max-h-[95vh] w-full max-w-2xl overflow-auto bg-paper p-6 md:p-9"><button className="icon-button absolute right-3 top-3" onClick={onClose}><X size={20} /></button><p className="eyebrow mb-3">Catalog editor</p><h2 className="font-serif text-4xl">{product ? "Edit product" : "Add product"}</h2><form className="mt-6 grid gap-4 sm:grid-cols-2" onSubmit={submit}><FormField label="Name" name="name" value={product?.name} /><FormField label="Slug" name="slug" value={product?.slug} pattern="[a-z0-9-]+" /><FormField label="Category" name="category" value={product?.category} /><FormField label="Price (IDR)" name="price_cents" value={product?.price_cents} type="number" min="0" /><FormField label="Stock" name="stock" value={product?.stock} type="number" min="0" /><FormField label="Image URL" name="image_url" value={product?.image_url} type="url" required={false} /><label className="grid gap-2 text-[9px] uppercase tracking-widest sm:col-span-2">Description<textarea className="field min-h-24" name="description" defaultValue={product?.description} required /></label><label className="flex items-center gap-2 text-xs"><input name="featured" type="checkbox" defaultChecked={product?.featured} /> Featured</label><label className="flex items-center gap-2 text-xs"><input name="active" type="checkbox" defaultChecked={product ? product.active : true} /> Visible</label><p className="text-xs text-rust sm:col-span-2">{error}</p><button className="button-dark sm:col-span-2" disabled={busy}>{busy ? "Saving..." : "Save product"}</button></form></section></div>;
}

function FormField({ label, name, value, type = "text", required = true, ...props }) {
  return <label className="grid gap-2 text-[9px] uppercase tracking-widest">{label}<input className="field" name={name} type={type} defaultValue={value ?? ""} required={required} {...props} /></label>;
}

function AdminOrderModal({ detail, onClose }) {
  return <div className="fixed inset-0 z-50 grid place-items-center bg-ink/50 p-4 backdrop-blur-sm"><section className="relative w-full max-w-xl bg-paper p-8"><button className="icon-button absolute right-3 top-3" onClick={onClose}><X size={20} /></button><p className="eyebrow mb-3">Fulfillment detail</p>{detail.loading ? <h2 className="font-serif text-3xl">Loading...</h2> : detail.error ? <p className="text-rust">{detail.error}</p> : <><h2 className="font-serif text-4xl">Order #{detail.order.id.slice(0, 8)}</h2><p className="mt-2 text-xs text-ink/45">{detail.order.customer_name} · {detail.order.customer_email}</p><p className="mt-1 text-xs capitalize text-ink/45">{detail.order.payment_method.replaceAll("_", " ")} · {detail.order.payment_status}</p><div className="mt-6">{detail.items.map((item) => <div className="flex justify-between border-t border-ink/15 py-4" key={item.product_id}><div><strong className="font-serif text-lg">{item.product_name}</strong><p className="text-[10px] text-ink/45">{item.quantity} × {money(item.unit_price_cents)}</p></div><strong>{money(item.subtotal_cents)}</strong></div>)}</div><div className="grid gap-2 border-t border-ink pt-5 text-xs"><div className="flex justify-between"><span>Subtotal</span><span>{money(detail.order.subtotal_cents)}</span></div><div className="flex justify-between"><span>Delivery</span><span>{detail.order.shipping_cents ? money(detail.order.shipping_cents) : "Complimentary"}</span></div><div className="mt-2 flex justify-between font-serif text-2xl"><span>Total</span><strong>{money(detail.order.total_cents)}</strong></div></div></>}</section></div>;
}

function PanelHeader({ title, children }) { return <header className="mb-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"><h2 className="font-serif text-3xl">{title}</h2>{children}</header>; }
function SearchInput({ value, onChange, placeholder }) { return <input className="border-b border-ink/20 bg-transparent px-2 py-2 text-xs outline-none focus:border-ink" value={value} onChange={(event) => onChange(event.target.value)} placeholder={placeholder} />; }
