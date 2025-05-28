use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash, 
    fmt 
};


/// Type-based extension storage, typically used by middleware
/// Each type can have exactly one value
pub struct Params { 
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
} 

impl Params { 
    //
    // Type-based params methods (for middleware)
    //
    /// Stores a value in the type-based params storage.
    /// Any previous value of the same type will be replaced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Params; 
    /// 
    /// let mut req = Params::default();
    ///
    /// // Store authentication information
    /// req.set(User { id: 123, name: "Alice".to_string() }); 
    /// ```
    pub fn set<T: 'static + Send + Sync>(&mut self, value: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(value));
    } 

    /// Retrieves a reference to a value from the type-based params storage.
    /// Returns `None` if no value of this type has been stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Params; 
    /// // In an authentication middleware 
    /// 
    /// // Set the req 
    /// let mut req = Params::default(); 
    /// req.set(User { id: 123, name: "Alice".to_string() }); 
    /// 
    /// if let Some(user) = req.get::<User>() {
    ///     println!("Request by: {}", user.name);
    ///     // Proceed with authenticated user
    /// } else {
    ///     return HttpResponse::unauthorized();
    /// }
    /// ```
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    } 

    /// Retrieves a mutable reference to a value from the type-based params storage.
    /// Returns `None` if no value of this type has been stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Params; 
    /// 
    /// // Set the req 
    /// let mut req = Params::default(); 
    /// req.set(1u8); 
    /// 
    /// // Update the u8 
    /// if let Some(number) = req.get_mut::<u8>() {
    ///     *number += 1 
    /// }
    /// ```
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    } 

    /// Removes a value from the type-based params storage and returns it.
    /// Returns `None` if no value of this type has been stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Params; 
    /// 
    /// // Set the req 
    /// let mut req = Params::default(); 
    /// req.set("Some String".to_string()); 
    /// 
    /// // Take ownership of a value
    /// if let Some(token) = req.take::<String>() {
    ///     drop(token) 
    /// }
    /// ```
    pub fn take<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    } 
}  

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params(types=[")?;
        for (i, ty) in self.inner.keys().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", ty)?;
        }
        write!(f, "])")
    }
} 

impl fmt::Debug for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let types: Vec<_> = self.inner.keys().collect();
        f.debug_struct("Params")
         .field("types", &types)
         .finish()
    }
} 

impl Default for Params { 
    fn default() -> Self {
        Self { 
            inner: HashMap::new() 
        }
    }
}

/// String-based extension storage, typically used by application code
/// Multiple values of the same type can be stored with different keys 
pub struct Locals { 
    inner: HashMap<String, Box<dyn Any + Send + Sync>> 
} 

impl Locals { 
    //
    // String-based locals methods (for application code)
    //
    /// Stores a value in the string-based locals storage with the given key.
    /// Any previous value with the same key will be replaced.
    ///
    /// # Examples
    ///
    /// ```rust 
    /// use starberry_core::extensions::Locals;  
    /// 
    /// let mut req = Locals::default();
    ///
    /// // Store various data with descriptive keys
    /// req.set("user_id", 123);
    /// req.set("is_premium", true);
    /// req.set("cart_items", vec!["item1", "item2"]);
    /// ```
    pub fn set<T: 'static + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.inner.insert(key.into(), Box::new(value));
    } 

    /// Retrieves a reference to a value from the string-based locals storage by key.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Locals;  
    /// 
    /// let mut req = Locals::default(); 
    /// 
    /// // Store various data with descriptive keys
    /// req.set("user_id", 123);
    /// req.set("is_premium", true);
    /// req.set("cart_items", vec!["item1", "item2"]); 
    ///     
    /// // In a request handler
    /// if let Some(is_premium) = req.get::<bool>("is_premium") {
    ///     if *is_premium {
    ///         // Show premium content
    ///     }
    /// } 
    /// 
    /// // With different types
    /// let user_id = req.get::<i32>("user_id");
    /// let items = req.get::<Vec<String>>("cart_items");
    /// ```
    pub fn get<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.inner
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    } 

    /// Retrieves a mutable reference to a value from the string-based locals storage by key.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Locals;  
    /// 
    /// let mut req = Locals::default(); 
    /// 
    /// // Modify a list of items 
    /// req.set("cart_items", vec!["item1", "item2"]); 
    /// if let Some(items) = req.get_mut::<Vec<String>>("cart_items") {
    ///     items.push("new_item".to_string());
    /// }
    /// ```
    pub fn get_mut<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.inner
            .get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    } 

    /// Removes a value from the string-based locals storage and returns it.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Locals;  
    /// 
    /// let mut req = Locals::default();  
    /// 
    /// // Set the token 
    /// req.set("session_token", "some_token".into_string()); 
    /// 
    /// // Take ownership of a value
    /// if let Some(token) = req.take::<String>("session_token") {
    ///     // Use and consume the token 
    ///     drop(token) 
    /// }
    /// ```
    pub fn take<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<T> {
        if let Some(boxed_any) = self.inner.get(key) {
            if boxed_any.downcast_ref::<T>().is_some() {
                // Type matches, now remove and downcast
                let any_box = self.inner.remove(key).unwrap();
                return any_box.downcast::<T>().ok().map(|boxed_t| *boxed_t);
            }
        }
        None 
    } 

    /// Returns all keys currently stored in the locals map
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::Locals;  
    /// 
    /// let mut req = Locals::default();
    ///
    /// // Store various data with descriptive keys
    /// req.set("user_id", 123);
    /// req.set("is_premium", true);
    /// req.set("cart_items", vec!["item1", "item2"]); 
    /// 
    /// // Inspect what data is attached to the request
    /// for key in req.keys() {
    ///     println!("Request has data with key: {}", key);
    /// }
    /// ```
    pub fn keys(&self) -> Vec<&str> {
        self.inner.keys().map(|s| s.as_str()).collect()
    } 

    //
    // Utility bridging methods
    //
    /// Exports a param value to the locals storage with the given key.
    /// The value must implement Clone. Does nothing if the param doesn't exist. 
    /// ```
    pub fn export_param<T: 'static + Clone + Send + Sync>(&mut self, params: &Params, key: impl Into<String>) {
        if let Some(value) = params.get::<T>() {
            let cloned = value.clone();
            self.set(key, cloned);
        }
    } 

    /// Imports a local value into the params storage.
    /// The value must implement Clone. Does nothing if the local doesn't exist. bv 
    pub fn import_param<T: 'static + Clone + Send + Sync>(&mut self, params: &mut Params, key: &str) {
        if let Some(value) = self.get::<T>(key) {
            let cloned = value.clone();
            params.set(cloned);
        }
    } 
} 

impl fmt::Display for Locals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<&str> = self.inner.keys().map(|k| k.as_str()).collect();
        write!(f, "Locals(keys={:?})", keys)
    }
} 

impl fmt::Debug for Locals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<&str> = self.inner.keys().map(|k| k.as_str()).collect();
        f.debug_struct("Locals")
         .field("keys", &keys)
         .finish()
    }
} 

impl Default for Locals { 
    fn default() -> Self {
        Self { 
            inner: HashMap::new() 
        }
    }
} 

/// Object-safe supertrait for stored values.
///
/// This trait is implemented for all types that are `Any + Clone + Send + Sync + 'static`.
/// It allows cloning a boxed trait object via dynamic dispatch. 
pub trait ParamValue: Any + Send + Sync + 'static {
    /// Clones this value and returns it as a boxed `ParamValue`.
    ///
    /// This enables cloning trait objects through dynamic dispatch.
    fn clone_box(&self) -> Box<dyn ParamValue>;

    /// Returns a reference to the underlying value as `Any`.
    ///
    /// Useful for downcasting to a concrete type.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the underlying value as `Any`.
    ///
    /// Useful for downcasting and in-place mutation.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> ParamValue for T
where
    T: Any + Clone + Send + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn ParamValue> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
} 

/// Type-based extension storage, typically used by middleware.
///
/// Stores at most one value per type. Internally uses a `HashMap<TypeId, Box<dyn ParamValue>>`.
pub struct ParamsClone {
    inner: HashMap<TypeId, Box<dyn ParamValue>>,
}

impl ParamsClone {
    /// Creates a new, empty `ParamsClone` container.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Stores a value of type `T` in the container.
    ///
    /// Any previous value of the same type will be replaced.
    /// `T` must implement `ParamValue` (i.e., `Any + Clone + Send + Sync + 'static`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::ParamsClone;
    ///
    /// let mut req = ParamsClone::default();
    ///
    /// // Store authentication information
    /// req.set(User { id: 123, name: "Alice".to_string() });
    /// ```
    pub fn set<T: ParamValue>(&mut self, value: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Retrieves a reference to a stored value of type `T`.
    /// Returns `None` if no such value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::ParamsClone;
    /// // In an authentication middleware
    ///
    /// let mut req = ParamsClone::default();
    /// req.set(User { id: 123, name: "Alice".to_string() });
    /// if let Some(user) = req.get::<User>() {
    ///     println!("Request by: {}", user.name);
    /// } else {
    ///     return HttpResponse::unauthorized();
    /// }
    /// ```
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &dyn Any).downcast_ref::<T>())
    }

    /// Retrieves a mutable reference to a stored value of type `T`.
    /// Returns `None` if no such value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::ParamsClone;
    ///
    /// let mut req = ParamsClone::default();
    /// req.set(1u8);
    ///
    /// // Update the u8
    /// if let Some(number) = req.get_mut::<u8>() {
    ///     *number += 1;
    /// }
    /// ```
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut dyn Any).downcast_mut::<T>())
    }

    /// Removes and returns a stored value of type `T`.
    /// Returns `None` if no such value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::ParamsClone;
    ///
    /// let mut req = ParamsClone::default();
    /// req.set("Some String".to_string());
    ///
    /// // Take ownership of a value
    /// if let Some(token) = req.take::<String>() {
    ///     drop(token)
    /// }
    /// ```
    pub fn take<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| {
                let any_box: Box<dyn Any> = boxed;
                any_box.downcast().ok()
            })
            .map(|boxed| *boxed)
    } 

    /// Combines entries from `other` into `self`.
    ///
    /// For each type not already present in `self`, clones the boxed value from `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::ParamsClone;
    ///
    /// let mut a = ParamsClone::default();
    /// a.set(1u8);
    /// let mut b = ParamsClone::default();
    /// b.set(2u8);
    /// a.combine(&b);
    /// assert_eq!(a.get::<u8>(), Some(&1u8));
    /// ```
    pub fn combine(&mut self, other: &Self) {
        for (ty, value) in &other.inner {
            self.inner.entry(*ty).or_insert((**value).clone_box());
        }
    }
} 

impl Clone for ParamsClone {
    fn clone(&self) -> Self { 
        let mut output = ParamsClone::default(); 
        output.combine(self); 
        output 
    }
} 

impl fmt::Display for ParamsClone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParamsClone(types=[")?;
        for (i, ty) in self.inner.keys().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", ty)?;
        }
        write!(f, "])")
    }
} 

impl fmt::Debug for ParamsClone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let types: Vec<_> = self.inner.keys().collect();
        f.debug_struct("ParamsClone")
         .field("types", &types)
         .finish()
    }
} 

impl Default for ParamsClone {
    fn default() -> Self {
        Self::new()
    }
}

/// String-based extension storage, typically used by application code.
///
/// Stores multiple values of the same type under different string keys.
/// Internally uses a `HashMap<String, Box<dyn Any + Send + Sync>>`.
pub struct LocalsClone {
    inner: HashMap<String, Box<dyn ParamValue>>,
}

impl LocalsClone {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Stores a value in the string-based locals storage with the given key.
    /// Any previous value with the same key will be replaced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::LocalsClone;
    ///
    /// let mut req = LocalsClone::default();
    ///
    /// // Store various data with descriptive keys
    /// req.set("user_id", 123);
    /// req.set("is_premium", true);
    /// req.set("cart_items", vec!["item1", "item2"]);
    /// ```
    pub fn set<T: ParamValue>(&mut self, key: impl Into<String>, value: T) {
        self.inner.insert(key.into(), Box::new(value));
    }

    /// Retrieves a reference to a value from the string-based locals storage by key.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::LocalsClone;
    ///
    /// let mut req = LocalsClone::default();
    ///
    /// // Store various data with descriptive keys
    /// req.set("user_id", 123);
    /// req.set("is_premium", true);
    /// req.set("cart_items", vec!["item1", "item2"]);
    ///
    /// // In a request handler
    /// if let Some(is_premium) = req.get::<bool>("is_premium") {
    ///     if *is_premium {
    ///         // Show premium content
    ///     }
    /// }
    ///
    /// // With different types
    /// let user_id = req.get::<i32>("user_id");
    /// let items = req.get::<Vec<String>>("cart_items");
    /// ```
    pub fn get<T: ParamValue>(&self, key: &str) -> Option<&T> {
        self.inner
            .get(key)
            .and_then(|boxed| (&**boxed as &dyn Any).downcast_ref::<T>())
    }

    /// Retrieves a mutable reference to a value from the string-based locals storage by key.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::LocalsClone;
    ///
    /// let mut req = LocalsClone::default();
    ///
    /// // Modify a list of items
    /// req.set("cart_items", vec!["item1", "item2"]);
    /// if let Some(items) = req.get_mut::<Vec<String>>("cart_items") {
    ///     items.push("new_item".to_string());
    /// }
    /// ```
    pub fn get_mut<T: ParamValue>(&mut self, key: &str) -> Option<&mut T> {
        self.inner
            .get_mut(key)
            .and_then(|boxed| boxed.as_any_mut().downcast_mut::<T>())
    }

    /// Removes a value from the string-based locals storage and returns it.
    /// Returns `None` if no value with this key exists or if the type doesn't match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::LocalsClone;
    ///
    /// let mut req = LocalsClone::default();
    ///
    /// // Set the token
    /// req.set("session_token", "some_token".to_string());
    ///
    /// // Take ownership of a value
    /// if let Some(token) = req.take::<String>("session_token") {
    ///     // Use and consume the token
    ///     drop(token)
    /// }
    /// ```
    pub fn take<T: ParamValue + Clone>(&mut self, key: &str) -> Option<T> { 
        if let Some(val) = self.get::<T>(key) {
            // Clone the stored value…
            let cloned = (*val).clone();
            // …then remove the original entry
            self.inner.remove(key);
            Some(cloned)
        } else {
            None
        }
    }

    /// Returns all keys currently stored in the locals map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::LocalsClone;
    ///
    /// let mut req = LocalsClone::default();
    ///
    /// // Store various data with descriptive keys
    /// req.set("user_id", 123);
    /// req.set("is_premium", true);
    /// req.set("cart_items", vec!["item1", "item2"]);
    ///
    /// // Inspect what data is attached to the request
    /// for key in req.keys() {
    ///     println!("Request has data with key: {}", key);
    /// }
    /// ```
    pub fn keys(&self) -> Vec<&str> {
        self.inner.keys().map(String::as_str).collect()
    }

    /// Exports a param value to the locals storage with the given key.
    /// The value must implement Clone. Does nothing if the param doesn't exist.
    pub fn export_param<T: ParamValue + Clone>(&mut self, params: &ParamsClone, key: impl Into<String>) {
        if let Some(value) = params.get::<T>() { 
            self.set(key, (*value).clone());
        }
    }

    /// Imports a local value into the params storage.
    /// The value must implement Clone. Does nothing if the local doesn't exist.
    pub fn import_param<T: ParamValue + Clone>(&mut self, params: &mut ParamsClone, key: &str) {
        if let Some(value) = self.get::<T>(key) {
            params.set((*value).clone());
        }
    }

    /// Combines entries from `other` into `self`.
    ///
    /// For each key not already present in `self`, moves the boxed value from `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::extensions::LocalsClone;
    ///
    /// let mut a = LocalsClone::default();
    /// a.set("x", 1);
    /// let mut b = LocalsClone::default();
    /// b.set("y", 2);
    /// a.combine(b);
    /// assert_eq!(a.get::<i32>("x"), Some(&1));
    /// assert_eq!(a.get::<i32>("y"), Some(&2));
    /// ```
    pub fn combine(&mut self, other: &LocalsClone) {
        for (key, value) in &other.inner {
            self.inner.entry((*key).clone()).or_insert((**value).clone_box());
        }
    }
} 

impl Clone for LocalsClone {
    fn clone(&self) -> Self { 
        let mut output = LocalsClone::default(); 
        output.combine(self); 
        output 
    }
}  

impl fmt::Display for LocalsClone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<&str> = self.inner.keys().map(String::as_str).collect();
        write!(f, "LocalsClone(keys={:?})", keys)
    }
} 

impl fmt::Debug for LocalsClone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<&str> = self.inner.keys().map(String::as_str).collect();
        f.debug_struct("LocalsClone")
         .field("keys", &keys)
         .finish()
    }
} 

impl Default for LocalsClone {
    fn default() -> Self {
        Self::new()
    }
} 

/// Helper function to merge two `HashMap<K, V>` without overwriting existing entries in `a`.
///
/// For each `(key, value)` in `b`, if `key` is not present in `a`, inserts `value.clone()`.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use starberry_core::extensions::combine_hashmap;
///
/// let mut a: HashMap<String, i32> = HashMap::new();
/// a.insert("a".to_string(), 1);
/// let mut b = HashMap::new();
/// b.insert("b".to_string(), 2);
/// b.insert("a".to_string(), 9);
/// combine_hashmap(&mut a, &b);
/// assert_eq!(a.get("a"), Some(&1));
/// assert_eq!(a.get("b"), Some(&2));
/// ```
pub fn combine_hashmap<K, V>(a: &mut HashMap<K, V>, b: &HashMap<K, V>)
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    for (key, value) in b {
        a.entry(key.clone()).or_insert_with(|| value.clone());
    }
} 

// Tests for the `extensions` module
#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq)]
    struct User { id: u32, name: String }

    #[test]
    fn test_param_value_clone_box_and_as_any() {
        let x = 42u8;
        let boxed: Box<dyn ParamValue> = x.clone_box();
        let y = boxed.as_any().downcast_ref::<u8>().unwrap();
        assert_eq!(*y, 42u8);
    }

    #[test]
    fn test_params_set_get() {
        let mut p = ParamsClone::default();
        p.set(100i32);
        assert_eq!(p.get::<i32>(), Some(&100));
    }

    #[test]
    fn test_params_replace() {
        let mut p = ParamsClone::default();
        p.set(1i32);
        p.set(2i32);
        assert_eq!(p.get::<i32>(), Some(&2));
    }

    #[test]
    fn test_params_get_mut() {
        let mut p = ParamsClone::default();
        p.set(10i32);
        if let Some(v) = p.get_mut::<i32>() { *v += 5; }
        assert_eq!(p.get::<i32>(), Some(&15));
    }

    #[test]
    fn test_params_take() {
        let mut p = ParamsClone::default();
        p.set(String::from("hello"));
        let v = p.take::<String>().unwrap();
        assert_eq!(v, "hello");
        assert!(p.get::<String>().is_none());
    }

    #[test]
    fn test_params_combine() {
        let mut a = ParamsClone::default();
        a.set(1u8);
        let mut b = ParamsClone::default();
        b.set(2u8);
        a.combine(&b);
        assert_eq!(a.get::<u8>(), Some(&1));
    }

    #[test]
    fn test_locals_set_get() {
        let mut l = LocalsClone::default();
        l.set("foo", 123i32);
        assert_eq!(l.get::<i32>("foo"), Some(&123));
    }

    #[test]
    fn test_locals_get_mut_and_take() {
        let mut l = LocalsClone::default();
        l.set("vec", vec![1, 2, 3]);
        if let Some(v) = l.get_mut::<Vec<i32>>("vec") { v.push(4); }
        assert_eq!(l.get::<Vec<i32>>("vec"), Some(&vec![1, 2, 3, 4]));
        let v = l.take::<Vec<i32>>("vec").unwrap();
        assert_eq!(v, vec![1, 2, 3, 4]);
        assert!(l.get::<Vec<i32>>("vec").is_none());
    }

    #[test]
    fn test_locals_keys() {
        let mut l = LocalsClone::default();
        l.set("a", true);
        l.set("b", false);
        let mut keys = l.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_locals_combine() {
        let mut a = LocalsClone::default();
        a.set("x", 1i32);
        let mut b = LocalsClone::default();
        b.set("y", 2i32);
        a.combine(&b);
        assert_eq!(a.get::<i32>("x"), Some(&1));
        assert_eq!(a.get::<i32>("y"), Some(&2));
    }

    #[test]
    fn test_combine_hashmap() {
        let mut a: HashMap<String, i32> = HashMap::new();
        a.insert("key1".to_string(), 1);
        let mut b = HashMap::new();
        b.insert("key2".to_string(), 2);
        b.insert("key1".to_string(), 9);
        combine_hashmap(&mut a, &b);
        assert_eq!(a.get("key1"), Some(&1));
        assert_eq!(a.get("key2"), Some(&2));
    }

    #[test]
    fn test_params_get_wrong_type_and_missing() {
        let mut p = Params::default();
        p.set(42u8);
        // Retrieving a type that was not set returns None
        assert!(p.get::<u16>().is_none());
        assert!(p.get_mut::<u16>().is_none());
        assert!(p.take::<u16>().is_none());

        // Retrieving the correct type works until taken
        assert_eq!(p.get::<u8>(), Some(&42u8));
        assert!(p.get_mut::<u8>().is_some());
        let val = p.take::<u8>().unwrap();
        assert_eq!(val, 42u8);
        assert!(p.get::<u8>().is_none());
    }

    #[test]
    fn test_locals_get_wrong_type_and_missing() {
        let mut l = Locals::default();
        l.set("foo", 123i32);
        // Wrong type or missing key returns None
        assert!(l.get::<u64>("foo").is_none());
        assert!(l.get::<i32>("bar").is_none());
        assert!(l.get_mut::<i32>("bar").is_none());
        assert!(l.get_mut::<u64>("foo").is_none());
        assert!(l.take::<u64>("foo").is_none());
        assert!(l.take::<i32>("bar").is_none());

        // Retrieving the correct type works until taken
        assert_eq!(l.get::<i32>("foo"), Some(&123));
        assert!(l.get_mut::<i32>("foo").is_some());
        let val = l.take::<i32>("foo").unwrap();
        assert_eq!(val, 123);
        assert!(l.get::<i32>("foo").is_none());
    } 
        #[test]
    fn test_paramsclone_clone_preserves_entries() {
        let mut original = ParamsClone::default();
        original.set(String::from("foo"));
        // Clone before any mutation
        let cloned = original.clone();
        // The cloned copy should have the same entry
        assert_eq!(cloned.get::<String>(), Some(&String::from("foo")));
        // Original still intact
        assert_eq!(original.get::<String>(), Some(&String::from("foo")));
    }

    #[test]
    fn test_paramsclone_clone_independence() {
        let mut original = ParamsClone::default();
        original.set(10u8);
        // Clone and then mutate the clone
        let mut cloned = original.clone();
        cloned.set(20u8);
        // Original remains unchanged
        assert_eq!(original.get::<u8>(), Some(&10u8));
        assert_eq!(cloned.get::<u8>(), Some(&20u8));
    }

    #[test]
    fn test_localsclone_clone_preserves_entries() {
        let mut original = LocalsClone::default();
        original.set("key", String::from("value"));
        // Clone before mutation
        let cloned = original.clone();
        // Both should contain the same data
        assert_eq!(cloned.get::<String>("key"), Some(&String::from("value")));
        assert_eq!(original.get::<String>("key"), Some(&String::from("value")));
    }

    #[test]
    fn test_localsclone_clone_independence() {
        let mut original = LocalsClone::default();
        original.set("counter", 1i32);
        // Clone and then change the clone
        let mut cloned = original.clone();
        cloned.set("counter", 2i32);
        // Original remains unaffected
        assert_eq!(original.get::<i32>("counter"), Some(&1));
        assert_eq!(cloned.get::<i32>("counter"), Some(&2));
    } 
} 