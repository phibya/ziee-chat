import { create } from "zustand";

interface AddProviderDrawerState {
  open: boolean;
  loading: boolean;
}

export const useAddProviderDrawerStore = create<AddProviderDrawerState>(() => ({
  open: false,
  loading: false,
}));

// Modal actions
export const openAddProviderDrawer = () => {
  useAddProviderDrawerStore.setState({
    open: true,
  });
};

export const closeAddProviderDrawer = () => {
  useAddProviderDrawerStore.setState({
    open: false,
    loading: false,
  });
};

export const setAddProviderDrawerLoading = (loading: boolean) => {
  useAddProviderDrawerStore.setState({
    loading,
  });
};
