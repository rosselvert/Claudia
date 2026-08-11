import { createContext, useContext, useEffect, useState } from "react";
import { api } from "./api";

const StoreContext = createContext(null);
let toastTimer;

export function StoreProvider({ children }) {
  const [user, setUser] = useState(null);
  const [sessionLoading, setSessionLoading] = useState(Boolean(localStorage.getItem("claudia_token")));
  const [cart, setCart] = useState({ items: [], item_count: 0, subtotal_cents: 0 });
  const [wishlist, setWishlist] = useState([]);
  const [addresses, setAddresses] = useState([]);
  const [authOpen, setAuthOpen] = useState(false);
  const [cartOpen, setCartOpen] = useState(false);
  const [toast, setToast] = useState("");

  function notify(message) {
    setToast(message);
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => setToast(""), 2600);
  }

  async function loadCustomerData() {
    const [nextCart, nextWishlist, nextAddresses] = await Promise.all([api("/cart"), api("/wishlist"), api("/addresses")]);
    setCart(nextCart);
    setWishlist(nextWishlist);
    setAddresses(nextAddresses);
  }

  useEffect(() => {
    if (!localStorage.getItem("claudia_token")) return;
    Promise.all([api("/me"), api("/cart"), api("/wishlist"), api("/addresses")])
      .then(([nextUser, nextCart, nextWishlist, nextAddresses]) => {
        setUser(nextUser); setCart(nextCart); setWishlist(nextWishlist); setAddresses(nextAddresses);
      })
      .catch(() => localStorage.removeItem("claudia_token"))
      .finally(() => setSessionLoading(false));
  }, []);

  async function authenticate(mode, values) {
    const response = await api(`/auth/${mode}`, { method: "POST", body: JSON.stringify(values) });
    localStorage.setItem("claudia_token", response.token);
    setUser(response.user);
    await loadCustomerData();
    setAuthOpen(false);
    notify(mode === "register" ? "Welcome to Claudia" : "Welcome back");
  }

  async function logout() {
    try { await api("/auth/logout", { method: "POST" }); } catch { /* Clear local session regardless. */ }
    localStorage.removeItem("claudia_token");
    setUser(null); setCart({ items: [], item_count: 0, subtotal_cents: 0 }); setWishlist([]); setAddresses([]);
  }

  async function addToCart(productId) {
    if (!user) { setAuthOpen(true); return false; }
    try {
      const nextCart = await api("/cart/items", { method: "POST", body: JSON.stringify({ product_id: productId, quantity: 1 }) });
      setCart(nextCart); notify("Added to your bag"); return true;
    } catch (error) { notify(error.message); return false; }
  }

  async function updateQuantity(productId, quantity) {
    try {
      const nextCart = await api(`/cart/items/${productId}`, { method: "PUT", body: JSON.stringify({ quantity }) });
      setCart(nextCart);
    } catch (error) { notify(error.message); }
  }

  async function removeCartItem(productId) {
    try {
      await api(`/cart/items/${productId}`, { method: "DELETE" });
      setCart((current) => {
        const items = current.items.filter((item) => item.product_id !== productId);
        return { items, item_count: items.reduce((sum, item) => sum + item.quantity, 0), subtotal_cents: items.reduce((sum, item) => sum + item.price_cents * item.quantity, 0) };
      });
    } catch (error) { notify(error.message); }
  }

  async function toggleWishlist(product) {
    if (!user) { setAuthOpen(true); return; }
    const saved = wishlist.some((item) => item.id === product.id);
    try {
      await api(`/wishlist/${product.id}`, { method: saved ? "DELETE" : "POST" });
      setWishlist((items) => saved ? items.filter((item) => item.id !== product.id) : [product, ...items]);
      notify(saved ? "Removed from wishlist" : "Saved to wishlist");
    } catch (error) { notify(error.message); }
  }

  async function saveAddress(values, addressId) {
    const saved = await api(addressId ? `/addresses/${addressId}` : "/addresses", {
      method: addressId ? "PUT" : "POST", body: JSON.stringify(values),
    });
    setAddresses((items) => {
      const next = items.some((item) => item.id === saved.id)
        ? items.map((item) => item.id === saved.id ? saved : item)
        : [saved, ...items];
      return saved.is_default
        ? next.map((item) => ({ ...item, is_default: item.id === saved.id }))
        : next;
    });
    notify(addressId ? "Address updated" : "Address saved");
    return saved;
  }

  async function deleteAddress(addressId) {
    try {
      await api(`/addresses/${addressId}`, { method: "DELETE" });
      setAddresses(await api("/addresses"));
      notify("Address removed");
    } catch (error) { notify(error.message); }
  }

  return <StoreContext.Provider value={{
    user, setUser, sessionLoading, cart, setCart, wishlist, addresses, authOpen, setAuthOpen,
    cartOpen, setCartOpen, toast, notify, authenticate, logout, addToCart,
    updateQuantity, removeCartItem, toggleWishlist, saveAddress, deleteAddress,
  }}>{children}</StoreContext.Provider>;
}

export function useStore() {
  return useContext(StoreContext);
}
